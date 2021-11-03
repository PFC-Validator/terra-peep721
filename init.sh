# code-id 11642 / 13373
#_init=$(cat init.json)
#json='{"name":"TerraPeeps","symbol":"PEEPS","minter":"terra1d85ncnvn822u5lul9kf8430dd3chyjd3ka2f98","public_key":"AibWLbKFlUwmGVzAcSfEk9ao60D3ba4Z8g3f166BGiJl","mint_amount":"3000000"}'
#echo ${_init}
json='{"name":"TerraPeeps","symbol":"PEEPS","minter":"terra1d85ncnvn822u5lul9kf8430dd3chyjd3ka2f98","public_key":"A8O7tqWAvsKW9XA7p2W8YZdIZmmadf9qoQmRiZq8xpvl","mint_amount":2000000,"max_issuance":10000}'
terrad tx wasm instantiate 16395 ${json} --chain-id bombay-12 --gas auto --from terrapeep --fees  5627uluna --admin terra1d85ncnvn822u5lul9kf8430dd3chyjd3ka2f98

#terrad tx wasm migrate terra1qdg78ja9xenjny6rn6nmtv2lj3u8jht0x3w5zd 13048 '{}' --from terrapeep --chain-id bombay-12 --fees 2640uluna
#terrad tx wasm migrate terra1qdg78ja9xenjny6rn6nmtv2lj3u8jht0x3w5zd 17460 '{}' --from terrapeep --chain-id bombay-12 --fees 2640uluna
terrad tx wasm migrate terra1m0rjzm27qetjj8fx89knnhl8frvlrmjcfultav 922 '{}'  --from terrapeep --chain-id bombay-12 --fees 2640uluna
terrad tx wasm migrate terra1t0l0sz0efnr7cm3hxked7nn2x7xx5syw02k8tc 922 '{}'  --from terrapeep --chain-id columbus-5 --fees 1ukrt
terrad tx wasm execute terra1m0rjzm27qetjj8fx89knnhl8frvlrmjcfultav  '{"set_public_key":{"public_key":"Ar5vm8QmL/RsBjSWaxgFizKhUrR4khjr4ax4wUgW4E2I"}}' --from terrapeep --chain-id bombay-12 --fees 2640uluna
