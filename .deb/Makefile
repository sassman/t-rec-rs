ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
deb:
	docker build -t rust-deb-build - < ${ROOT_DIR}/Dockerfile
	docker run --rm -it -v ${ROOT_DIR}/..:/usr/src/app -w /usr/src/app rust-deb-build cargo deb