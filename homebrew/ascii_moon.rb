class AsciiMoon < Formula
  desc "A TUI to show the moon phase"
  homepage "https://github.com/USERNAME/REPO"
  url "https://github.com/USERNAME/REPO/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_SHA256"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/ascii_moon", "--help"
  end
end
