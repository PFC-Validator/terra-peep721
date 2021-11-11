#terrad tx wasm store artifacts/terra_peep721.wasm --chain-id columbus-5 --from terrapeep --fees 18uusd --gas auto -y -b sync

terrad tx wasm store artifacts/terra_peep721.wasm --chain-id columbus-5 --from terrapeep --fees 352711uusd --gas auto -y -b sync --gas-adjustment 1.2

#terrad tx wasm store artifacts/terra_peep721.wasm --chain-id bombay-12 --from terrapeep --fees 391868uusd --gas auto -y -b sync --gas-adjustment 1.2

json='{"name":"TerraPeeps","symbol":"PEEPS","minter":"terra1d85ncnvn822u5lul9kf8430dd3chyjd3ka2f98","public_key":"A8O7tqWAvsKW9XA7p2W8YZdIZmmadf9qoQmRiZq8xpvl","mint_amount":2000000,"max_issuance":10000}'
terrad tx wasm instantiate 1138 ${json} --chain-id columbus-5 --gas auto --from terrapeep --fees  54627uusd --admin terra1d85ncnvn822u5lul9kf8430dd3chyjd3ka2f98 --gas-adjustment 1.2

#
#terra1t0l0sz0efnr7cm3hxked7nn2x7xx5syw02k8tc
