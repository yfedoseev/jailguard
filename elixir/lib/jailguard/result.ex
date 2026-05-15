defmodule JailGuard.Result do
  @moduledoc """
  Prompt injection detection result.
  """

  @type risk :: :safe | :low | :medium | :high | :critical

  @type t :: %__MODULE__{
          is_injection: boolean(),
          score: float(),
          confidence: float(),
          risk: risk()
        }

  defstruct [:is_injection, :score, :confidence, :risk]
end
