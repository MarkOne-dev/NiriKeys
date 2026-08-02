# Arch Linux

## Using AUR (Recommended)

The simplest way to install Noctalia on Arch Linux is through the Arch User Repository (AUR). This method installs the shell system-wide and handles dependencies automatically.

- AUR Helper
- Non-AUR Helper

*Please replace with your AUR helper of choice.*

**Terminal window**
```bash
paru -S noctalia-git
```

---

# Fedora

Noctalia is available from the default repos for Fedora 44 and up.

### Step 1: Install noctalia

**Terminal window**
```bash
sudo dnf install noctalia
```

An alternate version using git snapshots is available from the LionHeartP Copr repository.

### Step 1: Enable the repository

**Terminal window**
```bash
sudo dnf copr enable lionheartp/Hyprland
```

### Step 2: Install noctalia

**Terminal window**
```bash
sudo dnf install noctalia-git
```

---

# openSUSE

Noctalia is available via `home:neifua:Noctalia` repository on the openSUSE Build Service.

### Step 1: Add the OBS repository

- Tumbleweed
- Slowroll

**Terminal window**
```bash
sudo zypper addrepo --refresh --name noctalia-v5 https://download.opensuse.org/repositories/home:neifua:Noctalia/openSUSE_Tumbleweed/home:neifua:Noctalia.repo
```

### Step 2: Refresh and install noctalia:

- Stable (recommended)
- Unstable

**Terminal window**
```bash
sudo zypper refresh
sudo zypper install noctalia
```

> **Note**
> 
> v5 will only be available for Tumbleweed and Slowroll. Leap 16.0 is unable to build because sdbus-c++ is still on v1.6.0 while v5 requires >= v2.x.x
> 
> - **Stable version:** Updated when a new release is tagged
> - **Unstable version:** Automatically rebuilt on OBS whenever a new commit is pushed

---

# Gentoo

Noctalia is available in the GURU overlay.

> **Gentoo exception**
> 
> Gentoo packaging does not provide SLOTs for v4 and v5 as v4 will be dropped eventually, therefore they cannot be installed simultaneously.
> 
> Snapshots have been dropped in favor of official releases, so `=gui-apps/noctalia-5.0.0_pre99999999` is now superseded by `=gui-apps/noctalia-9999`.

**Step 1:** Enable and sync the overlay as read in Project:GURU/Information_for_End_Users.

**Step 2:** Unmask the `gui-apps/noctalia` package:

```text
gui-apps/noctalia **
```

**Step 3:** (Optional) Mask the live ebuild to use versioned releases

```text
<=gui-apps/noctalia-9999
```

**Step 4:** Emerge the package:

**Terminal window**
```bash
emerge --ask gui-apps/noctalia
```

---

# Void Linux

Noctalia is available through a custom XBPS repository.

### Step 1: Add the repository source

**Terminal window**
```bash
echo "repository=https://repo.voiders.dev" | sudo tee /etc/xbps.d/10-voiders-community.conf
```

### Step 2: Sync and install Noctalia

**Terminal window**
```bash
sudo xbps-install -S
sudo xbps-install noctalia
```

> **Note**
> 
> If Noctalia fails to run, install the sdbus-c++ package.

### Alternative: Build from source using xbps-src

```bash
# Install git, wget
sudo xbps-install git wget

# Set up the Void packages tree
git clone --depth 1 https://github.com/void-linux/void-packages.git
cd void-packages
./xbps-src binary-bootstrap

# Copy Noctalia build templates
mkdir srcpkgs/noctalia
wget https://git.voiders.dev/voiders-community/repository/raw/branch/main/pkgs/noctalia/template -P srcpkgs/noctalia/

# Build the packages
./xbps-src pkg noctalia

# Install the built packages
sudo xbps-install --repository=hostdir/binpkgs noctalia
```

---

# GNU Guix

The Git repository of Noctalia is also available as a Guix channel. It provides a `(noctalia)` module with the `noctalia-git` package.

```scheme
(channel
  (name 'noctalia)
  (url "https://github.com/noctalia-dev/noctalia")
  (branch "main"))
```

To use this channel, add it into your `~/.config/guix/channels.scm`. For example:

```scheme
(list (channel
        (name 'noctalia)
        (url "https://github.com/noctalia-dev/noctalia")
        (branch "main"))
      %default-guix-channel)
```

After updating channels via `guix pull`, Noctalia will be available to your Guix installation. Then you’re free to do what you want with the `noctalia-git` package ;-)

For example, spawn one-off software environment via `guix shell`:

**Terminal window**
```bash
guix shell noctalia-git
```

install it into your user profile via `guix package`:

**Terminal window**
```bash
guix install noctalia-git
```

declare it in your home environment (`...`s are placeholders for your existing configuration):

```scheme
(use-modules (noctalia)
             ...)

(home-environment
  (packages
   (list noctalia-git
         ...))
  ...)
```

or whole-system configuration:

```scheme
(use-modules (noctalia)
             ...)

(operating-system
  (packages
   (list noctalia-git
         ...))
  ...)
```

---

# Debian

Noctalia provides an APT repository for Debian-based distributions.

### Step 1: Install the repository signing key

**Terminal window**
```bash
wget https://pkg.noctalia.dev/deb/nickh-archive-keyring.deb && sudo dpkg -i nickh-archive-keyring.deb
```

### Step 2: Add the repository

- Debian Trixie
- Debian Sid
- Ubuntu 26.04

**Terminal window**
```bash
sudo wget -O /etc/apt/sources.list.d/noctalia-trixie.sources https://pkg.noctalia.dev/deb/noctalia-trixie.sources
```

### Step 3: Install noctalia

**Terminal window**
```bash
sudo apt update
sudo apt install noctalia
```

> **Note**
> 
> The APT repository provides amd64 and arm64 packages only.

---

# Manual Installation

> **Not Recommended**
> 
> Manual installation has no easy upgrade path. You will need to manually download and replace files for each update. We strongly recommend using a package manager (Arch, Fedora) whenever possible.

If you prefer to install Noctalia locally or want more control over the installation process, you can install it manually to your user configuration directory. This method works on any Linux distribution.

### Step 1: Sync and install dependencies

- Arch
- Fedora
- Debian/Ubuntu

**Terminal window**
```bash
sudo pacman -S meson gcc just \
  wayland wayland-protocols \
  libglvnd freetype2 fontconfig \
  cairo pango harfbuzz \
  libxkbcommon glib2 \
  libsecret libsodium \
  sdbus-cpp libpipewire wireplumber polkit \
  pam curl libwebp libjxl libsndfile librsvg \
  libqalculate libxml2 \
  md4c tomlplusplus \
  nlohmann-json stb \
  jemalloc
```

Vendored dependencies, with no system package needed: Wuffs, Luau, fzy, and Material Color Utilities.

System packages required beyond the Wayland/GL stack: `libwebp` handles WebP decoding and thumbnail encoding, `libjxl` handles JPEG XL decoding, `libsndfile` decodes shell sound effects (WAV, FLAC, Ogg/Vorbis, Opus, MP3, and AIFF), and Wuffs handles the other supported raster image formats.

Polkit agent support requires development files that provide the `polkit-agent-1` and `polkit-gobject-1` pkg-config modules. Some distros ship these in the runtime polkit package, while split-package distros use names such as `polkit-devel`, `polkit-dev`, or `libpolkit-agent-1-dev` / `libpolkit-gobject-1-dev`.

Optional: installing `jemalloc` (Fedora: `jemalloc-devel`, Arch: `jemalloc`, Debian/Ubuntu: `libjemalloc-dev`, Void: `jemalloc`) reduces memory fragmentation in long-running sessions. On glibc systems it is used automatically when detected. Use Meson’s `-Djemalloc=enabled` or `-Djemalloc=disabled` option to require or disable it explicitly.

Sanitizer runtime packages are only needed for ASan/UBSan builds configured with `just configure asan`.

### Step 2: Prepare directory

**Terminal window**
```bash
git clone https://github.com/noctalia-dev/noctalia --branch main
cd noctalia
```

### Step 3: Building and installing Noctalia

Requires `just` and `meson`.

- Release Mode
- Debug Mode

**Terminal window**
```bash
# Optimized release build in build-release/
just configure release
just build release

# Install the selected build mode. This does not build or reconfigure.
sudo just install release
```

Pass a prefix to configure to install somewhere other than `/usr/local`:

**Terminal window**
```bash
just configure release "$HOME/.local"
just build release
just install release
```

To remove files installed from a build directory, run `just uninstall release`.

Meson installs the binary and shipped assets using the normal prefix layout:

```text
/usr/local/bin/noctalia
/usr/local/share/noctalia/assets/...
```

Noctalia needs the shipped `assets/` tree at runtime. Copying only the `noctalia` binary is not enough.

Portable bundle layouts are also supported:

```text
bundle/
  noctalia
  assets/
```

```text
bundle/
  bin/noctalia
  share/noctalia/assets/
```

See `CONTRIBUTING.md` for the full runtime asset lookup order.
