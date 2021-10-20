FROM osrm/osrm-backend:v5.24.0

# 必要コマンドのインストール
RUN apt-get -q update && \
    apt-get -q install -y wget

# 地図情報のダウンロード（ここいじるとダウンロードやり直しになるので注意）
# NOTE: リンクをhttpsではなくhttpとしているのは、証明書がexpireしがちだから
#     : 参考： https://github.com/graphhopper/graphhopper/issues/1417
RUN mkdir /data && \
    wget --progress=bar:force:noscroll -O /data/map.osm.pbf http://download.geofabrik.de/asia/japan-latest.osm.pbf

# 地図データの前処理
COPY ./customized.lua /opt/customized.lua
RUN osrm-extract -p /opt/customized.lua /data/map.osm.pbf
RUN osrm-partition /data/map.osrm
RUN osrm-customize /data/map.osrm
