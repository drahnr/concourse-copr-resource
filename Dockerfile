FROM alpine:edge
COPY ./ /app
WORKDIR /app
RUN apk add --no-cache ca-certificates \
	&& apk add --no-cache curl \
	&& update-ca-certificates
RUN echo http://nl.alpinelinux.org/alpine/edge/testing >> /etc/apk/repositories \
	# llvm-libunwind is required to run the final rust binary, so we install it first
	&& apk add --no-cache llvm-libunwind \
	&& apk add --no-cache make openssl-dev openssl \
	# Next, we install rust and cargo and tag them in a virtual package called `.build-rust`
	&& apk add --no-cache --virtual .build-rust rust cargo \
	# Finally, we build our project
	&& cargo build --release \
	# After that we copy our binary to the project root (you need to adjust this to your project)
	&& cp target/release/concourse-copr-resource /usr/local/bin/ \
	&& mkdir -p /opt/resource \
	&& ln -s /usr/local/bin/concourse-copr-resource /opt/resource/check \
	&& ln -s /usr/local/bin/concourse-copr-resource /opt/resource/in \
	&& ln -s /usr/local/bin/concourse-copr-resource /opt/resource/out \
	# And discard the target/ directory so it won't bloat our image
	&& rm -rf target/ \
	# As the final cleanup step we uninstall our virtual package
	# This uninstalls cargo, rust and all dependencies that aren't needed anymore so they won't end up in the final image
	&& apk del --purge .build-rust \
	&& rm -rf /var/cache/apk/*
