ps aux | grep wrapper
ps aux | grep '[w]rapper' | awk '{print $2}' | xargs kill

echo "["$(ls -p | grep -v / | awk '{printf "\"%s\",", $0}')"]"
