import subprocess
import os

def get_version():
    try:
        result = subprocess.run(
            ['bash', '-c', 'echo $(git describe --tags --abbrev=0)r$(git rev-list $(git describe --tags --abbrev=0)..HEAD --count).$(git rev-parse --short=6 HEAD)'],
            capture_output=True,
            text=True, check=True
        )

        return result.stdout.strip().removeprefix("v")
    except subprocess.CalledProcessError:
        raise RuntimeError("Failed to get the latest git tag")

def create_pkgbuild(pkgname, pkgver, url):
    pkgbuild_content = f"""# Maintainer: Alexander Karpukhin <thealexdev23@gmail.com>

pkgname={pkgname}
pkgver={pkgver}
pkgrel=1
pkgdesc="A Web Renderer frontend for Power Options, a blazingly fast power management solution."
arch=('x86_64')
url={url}
license=('MIT')

depends=('power-options-daemon-git' 'webkit2gtk' 'xdotool')
makedepends=('cargo' 'dioxus-cli' 'git')

provides=('power-options-webview')
conflicts=('power-options-webview')

source=("git+https://github.com/thealexdev23/power-options.git")
sha256sums=('SKIP')

build() {{
  export RUSTUP_TOOLCHAIN=stable
  cd "$srcdir/power-options/crates/frontend-webview"
  dx build --release
}}

package() {{
  cd "$srcdir/power-options"

  install -Dm755 "target/release/frontend" "$pkgdir/usr/bin/power-options-webview"

  mkdir -p "$pkgdir/usr/lib/power-options-webview/"
  cp -r "crates/frontend-webview/assets" "$pkgdir/usr/lib/power-options-webview/"

  install -Dm755 "icon.png" "$pkgdir/usr/share/icons/power-options-webview.png"

  install -Dm755 "install/power-options-webview.desktop" "$pkgdir/usr/share/applications/power-options-webview.desktop"
}}
"""
    return pkgbuild_content

def main():
    pkgname = "power-options-webview-git"
    pkgver = get_version()
    url = "https://github.com/thealexdev23/power-options"

    pkgbuild_content = create_pkgbuild(pkgname, pkgver, url)

    os.makedirs('./pkgbuilds/webview-git', exist_ok=True)

    with open('./pkgbuilds/webview-git/PKGBUILD', 'w') as file:
        file.write(pkgbuild_content)

if __name__ == "__main__":
    main()
