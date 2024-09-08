import subprocess
import os

def get_latest_tag():
    try:
        result = subprocess.run(['git', 'describe', '--tags', '--abbrev=0'], capture_output=True, text=True, check=True)
        return result.stdout.strip().removeprefix("v")
    except subprocess.CalledProcessError:
        raise RuntimeError("Failed to get the latest git tag")

def create_pkgbuild(pkgname, pkgver, url):
    pkgbuild_content = f"""# Maintainer: Alexander Karpukhin <thealexdev23@gmail.com>

pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="A gtk frontend for Power Options, a blazingly fast power management solution."
arch=('x86_64')
url={url}
license=('MIT')

depends=('power-options-daemon' 'libadwaita' 'yad')
makedepends=('cargo')

provides=('power-options-gtk')
conflicts=('power-options-gtk-git' 'tlp' 'auto-cpufreq' 'power-profiles-daemon' 'cpupower-gui')

source=("$pkgname-$pkgver.tar.gz::{url}/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

prepare() {{
  export RUSTUP_TOOLCHAIN=stable
  cd "$srcdir/power-options/crates/frontend-gtk"
  cargo fetch --target "$(rustc -vV | sed -n 's/host: //p')"
}}

build() {{
  export RUSTUP_TOOLCHAIN=stable
  cd "$srcdir/power-options/crates/frontend-gtk"
  cargo build --frozen --release
}}

package() {{
  cd "$srcdir/power-options"

  install -Dm755 "target/release/frontend-gtk" "$pkgdir/usr/bin/power-options-gtk"
  install -Dm755 "icon.png" "$pkgdir/usr/share/icons/power-options.png"
  install -Dm755 "install/power-options-gtk.desktop" "$pkgdir/usr/share/applications/power-options-gtk.desktop"
}}
"""
    return pkgbuild_content

def main():
    pkgname = "power-options-gtk"
    pkgver = get_latest_tag()
    url = "https://github.com/thealexdev23/power-options"

    pkgbuild_content = create_pkgbuild(pkgname, pkgver, url)

    os.makedirs('./pkgbuilds/gtk', exist_ok=True)

    with open('./pkgbuilds/gtk/PKGBUILD', 'w') as file:
        file.write(pkgbuild_content)

if __name__ == "__main__":
    main()
