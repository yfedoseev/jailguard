defmodule JailGuard.Native do
  @moduledoc false

  mix_config = Mix.Project.config()
  version = mix_config[:version]

  use RustlerPrecompiled,
    otp_app: :jailguard,
    crate: "jailguard_nif",
    base_url: "https://github.com/yfedoseev/jailguard/releases/download/v#{version}",
    force_build: System.get_env("JAILGUARD_BUILD") in ["1", "true"],
    targets: ~w(
      aarch64-apple-darwin
      aarch64-unknown-linux-gnu
      x86_64-apple-darwin
      x86_64-pc-windows-msvc
      x86_64-unknown-linux-gnu
    ),
    version: version,
    nif_versions: ["2.16", "2.17"]

  def version, do: :erlang.nif_error(:nif_not_loaded)
  def download_model, do: :erlang.nif_error(:nif_not_loaded)
  def model_cache_dir, do: :erlang.nif_error(:nif_not_loaded)
  def detect(_text), do: :erlang.nif_error(:nif_not_loaded)
  def is_injection(_text), do: :erlang.nif_error(:nif_not_loaded)
  def score(_text), do: :erlang.nif_error(:nif_not_loaded)
  def detect_batch(_texts), do: :erlang.nif_error(:nif_not_loaded)
end
