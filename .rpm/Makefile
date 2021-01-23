ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
rpm:
	docker build -t rust-rpm-build - < ${ROOT_DIR}/Dockerfile
	docker run --rm -it -v ${ROOT_DIR}/..:/usr/src/app -w /usr/src/app rust-rpm-build cargo rpm build -v