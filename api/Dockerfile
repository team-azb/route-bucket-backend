FROM rust:1.53.0

# dbを待つためにmysqlコマンドをインストール
RUN apt update && \
    apt install -y default-mysql-client && \
    apt search caching-sha2-password

WORKDIR app

# SRTM elevation data
COPY resources/scripts .
RUN ./download_srtm_datas.sh

COPY . .

ENV TMP_DIR /tmp

# dockerに依存ライブラリのバイナリをキャッシュさせるよう教える
# これがないとコード変わるたびに毎回ライブラリのインストールから始まるらしい
# 参考: https://qiita.com/_mkazutaka/items/c4b602327c2ff7913718
# 普通に/app/target/にバイナリおくと何故か消えるので、
# 仮ディレクトリにコピーしてから戻す
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/app/target \
    --mount=type=cache,sharing=private,target=$TMP_DIR \
    cargo build --release && \
    mkdir -p $TMP_DIR && \
    cp -r ./target/* $TMP_DIR
# 元の場所に戻す
RUN mv -f $TMP_DIR/* ./target
