help:
	cat Makefile

run:
	cargo run watermelon.png watermelon.big.png

compress: *.png
	for f in *.png; do \
		pngquant --force --skip-if-larger --output "$$f" "$$f"; \
	done
