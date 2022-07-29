# Maintainer: Aakash Sharma <aakashsensharma@gmail.com>
# Contributor: Sergey A. <murlakatamenka@disroot.org>
# Contributor: rv178 <idliyout@gmail.com>

_pkgname="swhkd"
pkgname="${_pkgname}-git"
pkgver=.543.gf6ea9d6
pkgrel=2
arch=("x86_64")
url="https://github.com/waycrate/swhkd"
pkgdesc="A display server independent hotkey daemon inspired by sxhkd."
license=("BSD")
depends=("polkit")
makedepends=("rustup" "make" "git" "scdoc")
conflicts=("swhkd-musl-git")
source=("${_pkgname}::git+${url}.git")
sha256sums=("SKIP")

build(){
	cd "$_pkgname"
	make setup
	make
}

package() {
	cd "$_pkgname"
	install -Dm 755 ./target/release/swhkd "$pkgdir/usr/bin/swhkd"
	install -Dm 755 ./target/release/swhks "$pkgdir/usr/bin/swhks"

	install -Dm 644 -o root ./com.github.swhkd.pkexec.policy -t "$pkgdir/usr/share/polkit-1/actions"

	install -Dm 644 ./docs/swhkd.1.gz -t "$pkgdir/usr/share/man/man1/swhkd.1.gz"
	install -Dm 644 ./docs/swhkd.5.gz -t "$pkgdir/usr/share/man/man5/swhkd.5.gz"
	install -Dm 644 ./docs/swhks.1.gz -t "$pkgdir/usr/share/man/man1/swhks.1.gz"
	install -Dm 644 ./docs/swhkd-keys.5.gz -t "$pkgdir/usr/share/man/man5/swhkd-keys.5.gz"
}

pkgver() {
	cd $_pkgname
	echo "$(grep "^version =" Cargo.toml|head -n1|cut -d\" -f2|cut -d\- -f1).$(git rev-list --count HEAD).g$(git rev-parse --short HEAD)"
}
