FROM tiangolo/nginx-rtmp

RUN mkdir -p /etc/nginx/temp/tmp_hls
RUN mkdir -p /etc/nginx/logs
RUN touch /etc/nginx/logs/error.log
COPY nginx.conf /etc/nginx/nginx.conf