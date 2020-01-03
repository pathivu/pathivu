FROM frolvlad/alpine-glibc

COPY target/release/chola /bin/chola

CMD ["bin/chola"]