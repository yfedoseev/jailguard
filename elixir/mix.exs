defmodule JailGuard.MixProject do
  use Mix.Project

  @version "0.1.1"
  @source_url "https://github.com/yfedoseev/jailguard"

  def project do
    [
      app: :jailguard,
      version: @version,
      elixir: ">= 1.14.0",
      compilers: [:elixir_make] ++ Mix.compilers(),
      make_clean: ["clean"],
      make_error_message:
        "could not build the JailGuard Elixir NIF; ensure Rust, cargo, make, and a C compiler are installed",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: "Elixir bindings for JailGuard, a pure-Rust prompt-injection detector.",
      package: package(),
      docs: [
        main: "JailGuard",
        source_ref: "v#{@version}",
        source_url: @source_url
      ]
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:elixir_make, "~> 0.9", runtime: false}
    ]
  end

  defp package do
    [
      licenses: ["MIT", "Apache-2.0"],
      links: %{"GitHub" => @source_url},
      files: [
        "c_src",
        "lib",
        "test",
        "Makefile",
        "mix.exs",
        "README.md"
      ]
    ]
  end
end
