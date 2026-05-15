defmodule JailGuard.Error do
  @moduledoc """
  Error returned by JailGuard's native binding.
  """

  @type code ::
          :invalid_input
          | :download_failed
          | :inference_failed
          | :internal
          | :unknown

  @type t :: %__MODULE__{code: code(), message: String.t()}

  defexception [:code, :message]

  @spec new(code()) :: t()
  def new(code) do
    %__MODULE__{code: code, message: default_message(code)}
  end

  @impl Exception
  def message(%__MODULE__{message: message}), do: message

  defp default_message(:invalid_input), do: "jailguard: invalid input"
  defp default_message(:download_failed), do: "jailguard: ONNX model download failed"
  defp default_message(:inference_failed), do: "jailguard: inference / classification failed"
  defp default_message(:internal), do: "jailguard: internal error"
  defp default_message(:unknown), do: "jailguard: unknown native error"
end
