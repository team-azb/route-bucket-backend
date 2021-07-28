FROM ubuntu:21.10

RUN apt-get -q update && \
    apt-get -q install -y wget tar

# download mysqldef (https://github.com/k0kubun/sqldef#distributions)
RUN wget -O - https://github.com/k0kubun/sqldef/releases/latest/download/mysqldef_linux_amd64.tar.gz \
    | tar xvz

RUN mv mysqldef ./bin/

COPY schema.sql mysql/schema.sql