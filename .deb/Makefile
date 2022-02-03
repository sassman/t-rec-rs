ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
deb:
	docker run --rm -v ${ROOT_DIR}/..:/usr/src/app \
		-w /usr/src/app \
		5422m4n/rust-deb-builder:1.57.0