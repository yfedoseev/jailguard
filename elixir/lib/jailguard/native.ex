defmodule JailGuard.Native do
  @moduledoc false

  @on_load :load_nif

  def load_nif do
    path =
      :jailguard
      |> :code.priv_dir()
      |> :filename.join(~c"jailguard_nif")

    :erlang.load_nif(path, 0)
  end

  def version, do: :erlang.nif_error(:nif_not_loaded)
  def download_model, do: :erlang.nif_error(:nif_not_loaded)
  def model_cache_dir, do: :erlang.nif_error(:nif_not_loaded)
  def detect(_text), do: :erlang.nif_error(:nif_not_loaded)
  def is_injection(_text), do: :erlang.nif_error(:nif_not_loaded)
  def score(_text), do: :erlang.nif_error(:nif_not_loaded)
  def detect_batch(_texts), do: :erlang.nif_error(:nif_not_loaded)
end
