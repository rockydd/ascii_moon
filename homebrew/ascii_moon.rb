class AsciiMoon < Formula
  desc "A TUI to show the moon phase"
  homepage "https://github.com/rockydd/ascii_moon"
  url "https://github.com/rockydd/ascii_moon/archive/refs/tags/v0.2.1.tar.gz"
  sha256 "26dcfd49c1fceed69d25a973cdbb7fcb1fe520f20dd9d6dc8c9f10ee415e0ea7"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/ascii_moon", "--help"
  end
end