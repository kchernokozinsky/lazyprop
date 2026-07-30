# Homebrew formula for lazyprop.
#
# Publish this to a tap so users can `brew install kchernokozinsky/tap/lazyprop`:
#   1. Create a repo named `homebrew-tap` under your account.
#   2. Copy this file to `Formula/lazyprop.rb` in that repo.
#   3. After each release, update `version` and the three `sha256` values with
#      the checksums of the release archives (the Release workflow prints them,
#      or run `shasum -a 256 <archive>`).
#
# The formula installs a prebuilt binary and pulls in a Java runtime, which the
# embedded Secure Properties Tool needs at runtime.
class Lazyprop < Formula
  desc "Terminal UI for MuleSoft secure configuration properties"
  homepage "https://github.com/kchernokozinsky/lazyprop"
  version "0.1.0"
  license "MIT"

  depends_on "openjdk"

  on_macos do
    on_arm do
      url "https://github.com/kchernokozinsky/lazyprop/releases/download/v#{version}/lazyprop-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_AARCH64_APPLE_DARWIN_SHA256"
    end
    on_intel do
      url "https://github.com/kchernokozinsky/lazyprop/releases/download/v#{version}/lazyprop-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_X86_64_APPLE_DARWIN_SHA256"
    end
  end

  on_linux do
    url "https://github.com/kchernokozinsky/lazyprop/releases/download/v#{version}/lazyprop-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "REPLACE_WITH_X86_64_LINUX_SHA256"
  end

  def install
    bin.install "lazyprop"
  end

  test do
    assert_match "lazyprop v#{version}", shell_output("#{bin}/lazyprop --version")
  end
end
