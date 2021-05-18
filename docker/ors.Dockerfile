FROM openrouteservice/openrouteservice:v6.4.3

# 地図情報のダウンロード（ここいじるとダウンロードやり直しになるので注意）
RUN wget --progress=bar:force:noscroll -O /ors-core/data/osm_file.pbf https://download.geofabrik.de/asia/japan/kanto-latest.osm.pbf

ENV SAMPLE_PATH=openrouteservice/src/main/resources/app.config.sample
RUN head $SAMPLE_PATH
RUN jq '.[1].ors.info.base_url'