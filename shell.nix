{ pkgs }:

{
  buildInputs = with pkgs; [
    tectonic
    poppler_utils
  ];
}
