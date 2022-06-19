FROM archlinux:latest

# Mirror and package updates
USER root
RUN pacman -Syu --noconfirm
# Sudo
USER root
RUN pacman -S --noconfirm sudo 

# VSCode User
USER root
RUN useradd --comment "VSCode DevContainer User" --groups "root,wheel" --create-home --shell /bin/zsh vscode

# Dev-Tools
USER root
RUN pacman -S --noconfirm base-devel 
RUN pacman -S --noconfirm git
RUN pacman -S --noconfirm gcc
RUN pacman -S --noconfirm clang
RUN pacman -S --noconfirm llvm
RUN pacman -S --noconfirm lldb

# Xorg-X11 (window support)
USER root
RUN pacman -S --noconfirm xorg

# ZSH & Oh-My-ZSH
USER root
RUN pacman -S --noconfirm zsh

USER root
RUN sh -c "$(curl -fsSL https://raw.github.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
USER vscode
RUN sh -c "$(curl -fsSL https://raw.github.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

# Rust
USER root
RUN pacman -S --noconfirm rustup

# VSCode: Rust
USER vscode
RUN rustup default stable

# Entry
USER vscode
ENTRYPOINT ["/bin/zsh"]
