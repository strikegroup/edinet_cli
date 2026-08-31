# 有価証券報告書の収集と解析

## 目的

有価証券報告書などから企業の事業計画（経営方針や課題等の抽出）を行い、LLM Agentから活用できる状態にする。

MAIPL2で必要になる予定なので、先行して準備しておきたい。

<img src="./321f1b84-ce7b-11f0-834b-066924efa36b.png" title="ce6a341e34a6a9e329571b1c9e579d8d.png" width="640">

## やること

* EDINETから各企業の有価証券報告書を収集する

* データ（PDF or XBRL）をパース(方法はLLMでもヒューリスティックでも可)し、項目ごとに抽出し保存する。特に事業計画の内容が含まれる「課題」「リスク」のデータが利用できる状態にする。

（参考）
https://www.buffett-code.com/company/9166/
https://www.buffett-code.com/company/3697/

* M&Aに関連する情報をサマリーとしてLLMにまとめさせて保存する。

* データ抽出＆要約処理が完了したら、社内システムにWebhookでデータを投げる

* LLM Agentから利用できるようにMCPサーバー等を用意する
  * 法人番号もしくは証券番号を指定してデータを取得できるようにする
  * 全文検索でデータを取得できるようにする

## ついでに検証したい（あとでも可）

* Workflowエンジン
   *  Temporal Workflow https://zenn.dev/layerx/articles/b5f6cf6e47221e 

