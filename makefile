ifdef OS
	TOOLCHAIN = +stable-i686-pc-windows-msvc
	BINARYNAME = PawnScraper.dll
	OUPUTNAME = PawnScraper.dll
	CP_RELEASE = cp .\target\release\$(BINARYNAME) .\test\plugins\$(OUPUTNAME)
	CP_DEBUG = cp .\target\debug\$(BINARYNAME) .\test\plugins\$(OUPUTNAME)
else
	ifeq ($(shell uname), Linux)
		TOOLCHAIN = +stable-i686-unknown-linux-gnu
		BINARYNAME = libPawnScraper.so
		OUPUTNAME = PawnScraper.so
		CP_RELEASE = cp target/release/$(BINARYNAME) test/plugins/$(OUPUTNAME)
		CP_DEBUG = cp target/debug/$(BINARYNAME) test/plugins/$(OUPUTNAME)
	endif
endif

release:
	cargo $(TOOLCHAIN) build --release
	$(CP_RELEASE)

debug:
	cargo $(TOOLCHAIN) build
	$(CP_DEBUG)

setup:
	cd test && mkdir plugins
	cd test && mkdir gamemodes
	sampctl package ensure
	sampctl package build
	cd test && sampctl server ensure

ensure:
	sampctl package ensure
	
run:
	sampctl package build
	cd test && sampctl server run
	
clean:
	cargo clean
