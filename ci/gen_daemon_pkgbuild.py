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
pkgdesc="The core daemon for Power Options, a blazingly fast power management solution."
arch=('x86_64')
url="{url}"
license=('MIT')

depends=('acpid' 'zsh' 'pciutils' 'usbutils')
optdepends=('xorg-xrandr: needed for screen settings' 'brightnessctl: needed for brightness settings' 'net-tools: needed to disable ethernet cards')
makedepends=('cargo')

provides=('power-options-daemon')
conflicts=('power-options-daemon-git')

source=("$pkgname-$pkgver.tar.gz::{url}/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

install="daemon.install"

prepare() {{
  export RUSTUP_TOOLCHAIN=stable
  cd "$srcdir/power-options-$pkgver/crates/power-daemon-mgr"
  cargo fetch --target "$(rustc -vV | sed -n 's/host: //p')"
}}

build() {{
  export RUSTUP_TOOLCHAIN=stable
  cd "$srcdir/power-options-$pkgver/crates/power-daemon-mgr"
  cargo build --frozen --release
}}

package() {{
  cd "$srcdir/power-options-$pkgver"

  install -Dm755 "target/release/power-daemon-mgr" "$pkgdir/usr/bin/power-daemon-mgr"

  # Generate files
  "$pkgdir/usr/bin/power-daemon-mgr" -v generate-base-files --path "$pkgdir" --program-path "/usr/bin/power-daemon-mgr"
}}

post_install() {{
  systemctl daemon-reload
  systemctl enable power-options.service
  systemctl start power-options.service
  systemctl restart acpid.service
}}

post_upgrade() {{
  systemctl daemon-reload
  systemctl restart power-options.service
  systemctl restart acpid.service
}}

post_remove() {{
  systemctl daemon-reload
}}
"""
    return pkgbuild_content

def create_install_file():
    return f"""post_install() {{
  systemctl daemon-reload
  systemctl restart acpid.service
  systemctl enable --now power-options.service
}}

post_upgrade() {{
  systemctl daemon-reload
  systemctl restart acpid.service
  systemctl restart power-options.service
}}

post_remove() {{
  systemctl daemon-reload
}}"""

def main():
    pkgname = "power-options-daemon"
    pkgver = get_latest_tag()
    url = "https://github.com/thealexdev23/power-options"

    pkgbuild_content = create_pkgbuild(pkgname, pkgver, url)
    install_file_content = create_install_file();

    os.makedirs('./pkgbuilds/daemon', exist_ok=True)

    with open('./pkgbuilds/daemon/PKGBUILD', 'w') as file:
        file.write(pkgbuild_content)
    with open('./pkgbuilds/daemon/daemon.install', 'w') as file:
        file.write(install_file_content)

if __name__ == "__main__":
    main()
