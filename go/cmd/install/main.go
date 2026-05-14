// install downloads the prebuilt JailGuard native FFI library and prints
// the CGO_* (or purego) environment variables needed to use it.
//
//	go run github.com/yfedoseev/jailguard/go/cmd/install@latest          # CGo staticlib
//	go run github.com/yfedoseev/jailguard/go/cmd/install@v0.1.0          # pinned version
//	go run github.com/yfedoseev/jailguard/go/cmd/install@latest -shared  # purego shared lib
//
// Assets are fetched from this repo's GitHub Releases. The installer
// validates SHA-256 checksums and writes the extracted library into the
// user cache directory (`os.UserCacheDir()`), under `jailguard/v<version>/`.
package main

import (
	"archive/tar"
	"compress/gzip"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"flag"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"runtime"
	"runtime/debug"
	"strings"
)

const (
	repoOwner = "yfedoseev"
	repoName  = "jailguard"
	// fallbackVersion is used when debug.ReadBuildInfo() does not yield
	// a usable module version (e.g. when running inside the source tree
	// without a module-pinned execution). Keep in sync with Cargo.toml.
	fallbackVersion = "0.1.1"
)

// supportedPlatforms lists every GOOS/GOARCH the release pipeline builds
// for, along with the asset-name slug used in the GitHub Release URL.
var supportedPlatforms = map[string]struct {
	asset string // tarball platform slug (hyphen-separated)
	dir   string // cache-dir platform slug (underscore-separated)
}{
	"linux/amd64":   {"linux-amd64", "linux_amd64"},
	"linux/arm64":   {"linux-arm64", "linux_arm64"},
	"darwin/amd64":  {"darwin-amd64", "darwin_amd64"},
	"darwin/arm64":  {"darwin-arm64", "darwin_arm64"},
	"windows/amd64": {"windows-amd64", "windows_amd64"},
}

func main() {
	var (
		version      = flag.String("version", "", "release version to download (default: module version or "+fallbackVersion+")")
		installDir   = flag.String("dir", "", "install directory (default: $UserCacheDir/jailguard/v<version>)")
		shared       = flag.Bool("shared", false, "download shared library (.so/.dylib/.dll) for the purego backend instead of the static library")
		skipChecksum = flag.Bool("skip-checksum", false, "skip SHA-256 verification (NOT recommended)")
		envOnly      = flag.Bool("env-only", false, "skip download; just print env vars for an existing install")
	)
	flag.Parse()

	ver := resolveVersion(*version)
	plat, ok := supportedPlatforms[runtime.GOOS+"/"+runtime.GOARCH]
	if !ok {
		die("unsupported platform: %s/%s", runtime.GOOS, runtime.GOARCH)
	}

	dest := *installDir
	if dest == "" {
		cache, err := os.UserCacheDir()
		if err != nil {
			die("locate UserCacheDir: %v", err)
		}
		dest = filepath.Join(cache, "jailguard", "v"+ver)
	}

	if !*envOnly {
		if err := download(ver, plat.asset, *shared, dest, *skipChecksum); err != nil {
			die("%v", err)
		}
	}

	printEnv(dest, plat.dir, *shared, ver)
}

// resolveVersion picks the version to download. Precedence:
//
//  1. --version flag
//  2. The module version of this installer binary (debug.ReadBuildInfo)
//  3. fallbackVersion
func resolveVersion(flagVer string) string {
	if flagVer != "" {
		return strings.TrimPrefix(flagVer, "v")
	}
	if info, ok := debug.ReadBuildInfo(); ok {
		// The module path is github.com/yfedoseev/jailguard/go; the
		// installer is a subpackage. Find the parent.
		for _, dep := range info.Deps {
			if strings.HasPrefix(dep.Path, "github.com/yfedoseev/jailguard") {
				return strings.TrimPrefix(dep.Version, "v")
			}
		}
		if v := info.Main.Version; v != "" && v != "(devel)" {
			return strings.TrimPrefix(v, "v")
		}
	}
	return fallbackVersion
}

// download fetches the appropriate tarball + .sha256, verifies, and
// extracts under dest.
func download(version, platSlug string, shared bool, dest string, skipChecksum bool) error {
	kind := "static"
	if shared {
		kind = "shared"
	}

	var assetName string
	if shared {
		assetName = fmt.Sprintf("jailguard-go-ffi-shared-%s.tar.gz", platSlug)
	} else {
		assetName = fmt.Sprintf("jailguard-go-ffi-%s.tar.gz", platSlug)
	}

	baseURL := fmt.Sprintf("https://github.com/%s/%s/releases/download/v%s/%s",
		repoOwner, repoName, version, assetName)

	fmt.Fprintf(os.Stderr, "→ downloading %s library v%s for %s\n", kind, version, platSlug)
	fmt.Fprintf(os.Stderr, "  %s\n", baseURL)

	tarball, err := fetch(baseURL)
	if err != nil {
		return fmt.Errorf("download tarball: %w", err)
	}

	if !skipChecksum {
		sum, err := fetch(baseURL + ".sha256")
		if err != nil {
			return fmt.Errorf("download checksum: %w", err)
		}
		want := parseSha256(string(sum))
		if want == "" {
			return fmt.Errorf("checksum file empty or malformed")
		}
		got := sha256Hex(tarball)
		if got != want {
			return fmt.Errorf("checksum mismatch: got %s, want %s", got, want)
		}
		fmt.Fprintf(os.Stderr, "  checksum OK (%s)\n", want[:12])
	}

	if err := os.MkdirAll(dest, 0o750); err != nil {
		return fmt.Errorf("create dest %s: %w", dest, err)
	}
	if err := extractTarGz(tarball, dest); err != nil {
		return fmt.Errorf("extract: %w", err)
	}

	fmt.Fprintf(os.Stderr, "✓ extracted to %s\n", dest)
	return nil
}

// fetch performs a single GET and returns the body. Follows redirects
// (default http.Client does).
func fetch(url string) ([]byte, error) {
	// #nosec G107 -- url is built from known-good constants + version
	// tag and only ever points at github.com/yfedoseev/jailguard releases.
	resp, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("GET %s: %s", url, resp.Status)
	}
	return io.ReadAll(resp.Body)
}

// parseSha256 extracts a 64-char hex digest from either a bare-hex file
// or a shasum(1)-style "<digest>  <filename>" line.
func parseSha256(body string) string {
	body = strings.TrimSpace(body)
	if body == "" {
		return ""
	}
	first := strings.Fields(body)[0]
	if len(first) == 64 {
		if _, err := hex.DecodeString(first); err == nil {
			return strings.ToLower(first)
		}
	}
	return ""
}

func sha256Hex(b []byte) string {
	sum := sha256.Sum256(b)
	return hex.EncodeToString(sum[:])
}

// extractTarGz unpacks an in-memory .tar.gz into dest, refusing to write
// any path that escapes dest via traversal (../).
func extractTarGz(data []byte, dest string) error {
	gz, err := gzip.NewReader(byteReader(data))
	if err != nil {
		return err
	}
	defer func() { _ = gz.Close() }()
	tr := tar.NewReader(gz)
	for {
		hdr, err := tr.Next()
		if errors.Is(err, io.EOF) {
			return nil
		}
		if err != nil {
			return err
		}
		// #nosec G305 -- path traversal is explicitly validated on the
		// next line before any filesystem write happens.
		target := filepath.Join(dest, hdr.Name)
		if !strings.HasPrefix(filepath.Clean(target)+string(os.PathSeparator), filepath.Clean(dest)+string(os.PathSeparator)) {
			return fmt.Errorf("refusing path traversal: %s", hdr.Name)
		}
		switch hdr.Typeflag {
		case tar.TypeDir:
			if err := os.MkdirAll(target, 0o750); err != nil {
				return err
			}
		case tar.TypeReg:
			if err := os.MkdirAll(filepath.Dir(target), 0o750); err != nil {
				return err
			}
			// Mask before the cast so the gosec G115 int64->uint32
			// overflow check is satisfied (9-bit mask cannot overflow).
			perm := os.FileMode(uint32(hdr.Mode & 0o777)) //nolint:gosec // G115 false positive after mask
			// #nosec G304 -- target is constrained to `dest` by the
			// HasPrefix check above; cannot escape via tar header.
			f, err := os.OpenFile(target, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, perm)
			if err != nil {
				return err
			}
			// #nosec G110 -- tarball is downloaded from a known
			// github.com release URL and verified against its SHA-256
			// checksum before reaching this code path.
			if _, err := io.Copy(f, tr); err != nil {
				_ = f.Close()
				return err
			}
			if err := f.Close(); err != nil {
				return err
			}
		}
	}
}

// byteReader returns an io.Reader over an in-memory slice without
// dragging bytes.Buffer into the dependency graph.
type byteSliceReader struct {
	data []byte
	pos  int
}

func byteReader(b []byte) *byteSliceReader { return &byteSliceReader{data: b} }

func (r *byteSliceReader) Read(p []byte) (int, error) {
	if r.pos >= len(r.data) {
		return 0, io.EOF
	}
	n := copy(p, r.data[r.pos:])
	r.pos += n
	return n, nil
}

// printEnv emits shell-friendly export lines to stdout. Stderr already
// has the progress messages; stdout is dedicated to env vars so users
// can `eval "$(go run …/cmd/install)"`.
func printEnv(dest, platDir string, shared bool, version string) {
	libDir := filepath.Join(dest, "lib", platDir)
	incDir := filepath.Join(dest, "include")

	fmt.Fprintf(os.Stderr, "\n# Add these to your shell profile or run them inline:\n")

	if shared {
		var libFile string
		switch runtime.GOOS {
		case "darwin":
			libFile = "libjailguard.dylib"
		case "windows":
			libFile = "jailguard.dll"
		default:
			libFile = "libjailguard.so"
		}
		fmt.Printf("export CGO_ENABLED=0\n")
		fmt.Printf("export JAILGUARD_LIB_PATH=%q\n", filepath.Join(libDir, libFile))
		return
	}

	// Static-link path. The ldflags are platform-specific because each
	// platform needs different system libs.
	staticLib := filepath.Join(libDir, "libjailguard.a")
	if runtime.GOOS == "windows" {
		staticLib = filepath.Join(libDir, "libjailguard.a")
	}
	fmt.Printf("export CGO_CFLAGS=%q\n", "-I"+incDir)

	switch runtime.GOOS {
	case "linux":
		fmt.Printf("export CGO_LDFLAGS=%q\n", staticLib+" -lm -lpthread -ldl -lrt -lgcc_s -lutil -lc")
	case "darwin":
		fmt.Printf("export CGO_LDFLAGS=%q\n", staticLib+" -framework CoreFoundation -framework Security")
	case "windows":
		fmt.Printf("export CGO_LDFLAGS=%q\n", staticLib+" -lws2_32 -luserenv -lntdll")
	default:
		fmt.Printf("export CGO_LDFLAGS=%q\n", staticLib)
	}
}

func die(format string, args ...any) {
	fmt.Fprintf(os.Stderr, "install: "+format+"\n", args...)
	os.Exit(1)
}
