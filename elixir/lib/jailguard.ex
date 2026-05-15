defmodule JailGuard do
  @moduledoc """
  Elixir bindings for JailGuard prompt-injection detection.

  The binding loads a Rustler NIF. On supported platforms the NIF is
  fetched as a precompiled artifact from the matching GitHub release;
  set `JAILGUARD_BUILD=1` to compile from source instead (requires a
  Rust toolchain).
  """

  alias JailGuard.{Error, Native, Result}

  @type text :: binary()

  @doc """
  Returns the linked JailGuard native library version.
  """
  @spec version() :: String.t()
  def version, do: Native.version()

  @doc """
  Pre-downloads the ONNX embedding model into the JailGuard cache.
  """
  @spec download_model() :: :ok | {:error, Error.t()}
  def download_model do
    case Native.download_model() do
      :ok -> :ok
      {:error, code} -> {:error, Error.new(code)}
    end
  end

  @doc """
  Returns the ONNX model cache directory.
  """
  @spec model_cache_dir() :: {:ok, String.t()} | {:error, Error.t()}
  def model_cache_dir do
    Native.model_cache_dir() |> normalize_value()
  end

  @doc """
  Detects whether `text` is a prompt injection.
  """
  @spec detect(text()) :: {:ok, Result.t()} | {:error, Error.t()}
  def detect(text) do
    with :ok <- validate_text(text) do
      Native.detect(text) |> normalize_value()
    end
  end

  @doc """
  Detects whether `text` is a prompt injection, raising on error.
  """
  @spec detect!(text()) :: Result.t()
  def detect!(text) do
    case detect(text) do
      {:ok, result} -> result
      {:error, error} -> raise error
    end
  end

  @doc """
  Returns `true` when `text` is classified as a prompt injection.
  """
  @spec is_injection(text()) :: {:ok, boolean()} | {:error, Error.t()}
  def is_injection(text) do
    with :ok <- validate_text(text) do
      Native.is_injection(text) |> normalize_value()
    end
  end

  @doc """
  Returns the raw injection probability score in `[0.0, 1.0]`.
  """
  @spec score(text()) :: {:ok, float()} | {:error, Error.t()}
  def score(text) do
    with :ok <- validate_text(text) do
      Native.score(text) |> normalize_value()
    end
  end

  @doc """
  Classifies a list of texts in input order.
  """
  @spec detect_batch([text()]) :: {:ok, [Result.t()]} | {:error, Error.t()}
  def detect_batch(texts) when is_list(texts) do
    with :ok <- validate_texts(texts) do
      Native.detect_batch(texts) |> normalize_value()
    end
  end

  def detect_batch(_texts), do: invalid_input()

  defp validate_text(text) when is_binary(text) do
    cond do
      not String.valid?(text) -> invalid_input()
      :binary.match(text, <<0>>) != :nomatch -> invalid_input()
      true -> :ok
    end
  end

  defp validate_text(_text), do: invalid_input()

  defp validate_texts(texts) do
    Enum.reduce_while(texts, :ok, fn text, :ok ->
      case validate_text(text) do
        :ok -> {:cont, :ok}
        {:error, error} -> {:halt, {:error, error}}
      end
    end)
  end

  defp normalize_value({:ok, value}), do: {:ok, value}
  defp normalize_value({:error, code}), do: {:error, Error.new(code)}

  defp invalid_input, do: {:error, Error.new(:invalid_input)}
end
