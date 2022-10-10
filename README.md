# wirewhale
[![ci](https://github.com/gawetto/wirewhale/actions/workflows/ci.yml/badge.svg)](https://github.com/gawetto/wirewhale/actions/workflows/ci.yml)

pcapフォーマットのデータをtuiで可視化するプログラムです。パケットのキャプチャ自体はtsharkやtcpdumpを利用してください。

## インストール方法
* 下記から実行したいOSのファイルをダウンロード・解凍してPATHを通してください。
`https://github.com/gawetto/wirewhale/releases/latest`

## 実行方法
### Ubuntu
#### pcapファイルを可視化する場合
```
cat pcap.pcap|wirewhale
```

#### リアルタイムでキャプチャしたデータを可視化する場合
* tcpdumpをインストールしてください
```
tcpdump -l -w - 2>/dev/null|wirewhale
```

### Windows
#### pcapファイルを可視化する場合
```
cmd /c "type pcap.pcap|wirewhale.exe"
```
※powershellではパイプでバイナリデータを渡せないのでcmdを使う必要があります

#### リアルタイムでキャプチャしたデータを可視化する場合
* wiresharkをインストールしてください
```
cmd /c '"C:\Program Files\Wireshark\tshark.exe" -F pcap -w - 2>nul|wirewhale.exe'
```
※powershellではパイプでバイナリデータを渡せないのでcmdを使う必要があります

