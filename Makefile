html:
	pandoc zprava.md -o zprava.html -css=pandoc.css
compile:
	cargo build --release
