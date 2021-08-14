# dbの起動待ち用スクリプト

echo "Waiting for the db to start."

until mysqladmin ping -h db --silent;
do
  sleep 10
  echo "Still waiting..."
done

echo "MySQL is up!"
