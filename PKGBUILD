# Maintainer: kosa <kosa@users.noreply.github.com>
pkgname=crtty
pkgver=0.1.0
pkgrel=1
pkgdesc="Post-processing shader framework for kitty terminal via LD_PRELOAD"
arch=('x86_64')
url="https://github.com/kosa/CRTty"
license=('MIT')
depends=('glibc')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

prepare() {
  cd "CRTty-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
  cd "CRTty-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  cargo build --frozen --release --workspace
}

package() {
  cd "CRTty-$pkgver"
  make DESTDIR="$pkgdir" PREFIX=/usr install
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 crtty.conf.example "$pkgdir/usr/share/doc/$pkgname/crtty.conf.example"
}
