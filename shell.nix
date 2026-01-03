{pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  name = "Blonder";

  buildInputs = with pkgs; [
    cmake
    pkg-config
    wayland
    libxkbcommon
    libffi
    libx11
    libxrandr
    libxinerama
    libxcursor
    libxi
    libxext
    libGL
    libglvnd
    mesa
    vulkan-loader
  ];
  LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
    # Graphics/Display
    libGL
    libglvnd
    mesa
    vulkan-loader

    # X11
    libx11
    libxrandr
    libxinerama
    libxcursor
    libxi
    libxext
    
    # Wayland
    wayland
    libxkbcommon
    
    # Audio
    # alsa-lib
    # pipewire
    # libpulseaudio
    
    # System
    stdenv.cc.cc.lib
  ];

  shellHook = ''
    echo Hi
  '';
}
