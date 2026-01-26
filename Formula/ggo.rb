class Ggo < Formula
  desc "Smart git branch navigation with frecency-based ranking"
  homepage "https://github.com/XavierFabregat/ggo"
  version "0.2.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/XavierFabregat/ggo/releases/download/v0.2.2/ggo-macos-arm64.tar.gz"
      sha256 "PLACEHOLDER_MACOS_ARM64_SHA256"
    end
    on_intel do
      url "https://github.com/XavierFabregat/ggo/releases/download/v0.2.2/ggo-macos-amd64.tar.gz"
      sha256 "PLACEHOLDER_MACOS_AMD64_SHA256"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/XavierFabregat/ggo/releases/download/v0.2.2/ggo-linux-amd64.tar.gz"
      sha256 "PLACEHOLDER_LINUX_AMD64_SHA256"
    end
  end

  def install
    bin.install "ggo"
  end

  test do
    assert_match "ggo 0.2.2", shell_output("#{bin}/ggo --version")
  end
end
