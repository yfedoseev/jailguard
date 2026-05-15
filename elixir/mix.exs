defmodule JailGuard.MixProject do
  use Mix.Project

  @version "0.1.2"
  @source_url "https://github.com/yfedoseev/jailguard"

  def project do
    [
      app: :jailguard,
      version: @version,
      # rustler 0.36 and rustler_precompiled 0.8 require Elixir 1.15+.
      elixir: ">= 1.15.0",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: description(),
      package: package(),
      docs: docs(),
      name: "JailGuard"
    ]
  end

  def application do
    [extra_applications: [:logger]]
  end

  defp description do
    "Elixir bindings for JailGuard, a pure-Rust prompt-injection detector " <>
      "with a 1.5 MB embedded MLP classifier."
  end

  defp deps do
    [
      {:rustler_precompiled, "~> 0.8"},
      {:rustler, "~> 0.36", optional: true},
      {:ex_doc, "~> 0.34", only: :dev, runtime: false}
    ]
  end

  defp package do
    [
      licenses: ["MIT", "Apache-2.0"],
      links: %{
        "GitHub" => @source_url,
        "Changelog" => "#{@source_url}/blob/main/CHANGELOG.md"
      },
      files: ~w(
        lib
        native/jailguard_nif/src
        native/jailguard_nif/Cargo.toml
        checksum-Elixir.JailGuard.Native.exs
        .formatter.exs
        mix.exs
        README.md
      )
    ]
  end

  defp docs do
    [
      main: "JailGuard",
      source_ref: "v#{@version}",
      source_url: @source_url,
      extras: ["README.md"]
    ]
  end
end
