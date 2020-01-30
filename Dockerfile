FROM frolvlad/alpine-glibc

COPY target/release/pathivu /bin/pathivu

CMD ["bin/pathivu"]