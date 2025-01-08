# Inspired by https://hub.docker.com/r/tiangolo/nginx-rtmp/dockerfile

ARG NGINX_VERSION=1.27.3

FROM nginx:${NGINX_VERSION} AS build
ENV RTMP_MODULE_VERSION=1.2.2

RUN mkdir -p /tmp/build
RUN apt-get update && \
    apt-get install -y \
        ca-certificates \
        openssh-client \
        git \
        wget \
        libxml2 \
        libxslt1-dev \
        libpcre3 \
        libpcre3-dev \
        zlib1g \
        zlib1g-dev \
        openssl \
        libssl-dev \
        libtool \
        automake \
        gcc \
        g++ \
        make && \
    rm -rf /var/cache/apt

RUN wget "http://nginx.org/download/nginx-${NGINX_VERSION}.tar.gz" && \
    tar -C /tmp/build -xzvf nginx-${NGINX_VERSION}.tar.gz

    RUN wget "https://codeload.github.com/arut/nginx-rtmp-module/tar.gz/v${RTMP_MODULE_VERSION}" -O nginx-rtmp-module.tar.gz
RUN tar -C /tmp/build -xzvf nginx-rtmp-module.tar.gz

WORKDIR /tmp/build/nginx-${NGINX_VERSION}
RUN ./configure \
            --sbin-path=/usr/local/sbin/nginx \
            --conf-path=/etc/nginx/nginx.conf \
            --error-log-path=/var/log/nginx/error.log \
            --pid-path=/var/run/nginx/nginx.pid \
            --lock-path=/var/lock/nginx/nginx.lock \
            --http-log-path=/var/log/nginx/access.log \
            --http-client-body-temp-path=/tmp/nginx-client-body \
            --with-http_ssl_module \
            --with-threads \
            --with-ipv6 \
            --with-http_auth_request_module  \
            --add-module=/tmp/build/nginx-rtmp-module-${RTMP_MODULE_VERSION} && \
        make -j $(getconf _NPROCESSORS_ONLN) && \
        make install && \
        mkdir -p /var/lock/nginx && \
        rm -rf /tmp/build

RUN mkdir -p /etc/nginx/temp/tmp_hls
RUN mkdir -p /etc/nginx/logs
RUN touch /etc/nginx/logs/error.log

# Forward logs to Docker
RUN ln -sf /dev/stdout /var/log/nginx/access.log && \
    ln -sf /dev/stderr /var/log/nginx/error.log

COPY nginx.conf /etc/nginx/nginx.conf.template

COPY run-nginx.sh /usr/bin/run-nginx.sh
RUN chmod +x /usr/bin/run-nginx.sh

EXPOSE 1935
CMD ["/usr/bin/run-nginx.sh"]