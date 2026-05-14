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

  @doc false
  @spec from_native({boolean(), float(), float(), risk()}) :: t()
  def from_native({is_injection, score, confidence, risk}) do
    %__MODULE__{
      is_injection: is_injection,
      score: score,
      confidence: confidence,
      risk: risk
    }
  end
end
