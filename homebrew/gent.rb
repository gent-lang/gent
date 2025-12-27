# typed: false
# frozen_string_literal: true

# Homebrew formula for GENT
#
# To use this formula, you need to set up a Homebrew tap:
#   1. Create a repository: github.com/gent-lang/homebrew-gent
#   2. Copy this file to: Formula/gent.rb
#   3. Users can then install with: brew tap gent-lang/gent && brew install gent
#
# This formula is auto-updated by the release workflow.

class Gent < Formula
  desc "A programming language for AI agents"
  homepage "https://github.com/gent-lang/gent"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/gent-lang/gent/releases/download/v0.1.0/gent-v0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X64"
    end

    on_arm do
      url "https://github.com/gent-lang/gent/releases/download/v0.1.0/gent-v0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/gent-lang/gent/releases/download/v0.1.0/gent-v0.1.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX"
    end
  end

  def install
    bin.install "gent"
  end

  test do
    # Create a simple test file
    (testpath/"hello.gnt").write <<~EOS
      agent Hello {
        systemPrompt: "You are friendly."
        model: "gpt-4o-mini"
      }
    EOS

    # Check that help works
    assert_match "GENT", shell_output("#{bin}/gent --help")
  end
end
