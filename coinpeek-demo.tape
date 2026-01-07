Output coinpeek-demo.gif

Set Shell "bash"
Set FontSize 22
Set Width 1200
Set Height 800

# First show installation
Type "echo '🚀 Running CoinPeek - Real-time Crypto Monitor'" Enter
Type "echo 'This will show the actual TUI interface...'" Enter
Sleep 2

# Actually run CoinPeek
Type "cargo run --all-features" Enter
Sleep 8

# After timeout, show explanation
Type "echo ''" Enter
Type "echo '🎯 What you just saw:'" Enter
Type "echo '• Real-time cryptocurrency prices from Binance API'" Enter
Type "echo '• Freshness indicator showing sync status'" Enter
Type "echo '• Footer help hint: Press ? for help'" Enter
Type "echo '• Interactive navigation with keyboard/mouse'" Enter
Sleep 2

Type "echo ''" Enter
Type "echo 'Perfect first-run experience!'" Enter
