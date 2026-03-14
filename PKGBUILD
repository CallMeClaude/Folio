# Maintainer: CallMeClaude <https://github.com/CallMeClaude>
pkgname=folio-bin
pkgver=0.1.0
pkgrel=1
pkgdesc="A beautiful word processor for Linux"
arch=('x86_64')
url="https://github.com/CallMeClaude/Folio"
license=('GPL3')
depends=('gtk4' 'libadwaita' 'pango' 'cairo' 'glib2')
provides=('folio')
conflicts=('folio')
source=("folio::git+https://github.com/CallMeClaude/Folio.git")
sha256sums=('SKIP')

build() {
    cd "$srcdir/folio"
    cargo build --release -p folio-gtk
}

package() {
    cd "$srcdir/folio"

    # Binary
    install -Dm755 target/release/folio "$pkgdir/usr/bin/folio"

    # .desktop file
    install -Dm644 data/org.folio.Folio.desktop \
        "$pkgdir/usr/share/applications/org.folio.Folio.desktop"

    # Icons (install if present)
    for size in 16 32 48 64 128 256 512; do
        icon="data/icons/hicolor/${size}x${size}/apps/org.folio.Folio.png"
        if [ -f "$icon" ]; then
            install -Dm644 "$icon" \
                "$pkgdir/usr/share/icons/hicolor/${size}x${size}/apps/org.folio.Folio.png"
        fi
    done

    # SVG icon
    if [ -f "data/icons/hicolor/scalable/apps/org.folio.Folio.svg" ]; then
        install -Dm644 "data/icons/hicolor/scalable/apps/org.folio.Folio.svg" \
            "$pkgdir/usr/share/icons/hicolor/scalable/apps/org.folio.Folio.svg"
    fi

    # Bundled fonts
    for font in data/fonts/*.ttf data/fonts/*.otf; do
        [ -f "$font" ] && install -Dm644 "$font" \
            "$pkgdir/usr/share/folio/fonts/$(basename $font)"
    done

    # License
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
