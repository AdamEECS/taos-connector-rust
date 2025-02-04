use taos_query::{
    block_in_place_or_global,
    prelude::{AsAsyncConsumer, RawMeta, TBuilder, Timeout},
    RawBlock,
};

enum TmqBuilderInner {
    Native(crate::sys::TmqBuilder),
    Ws(taos_ws::consumer::TmqBuilder),
}

enum ConsumerInner {
    Native(crate::sys::Consumer),
    Ws(taos_ws::consumer::Consumer),
}

enum OffsetInner {
    Native(crate::sys::tmq::Offset),
    Ws(taos_ws::consumer::Offset),
}
enum MetaInner {
    Native(crate::sys::tmq::Meta),
    Ws(taos_ws::consumer::Meta),
}

enum DataInner {
    Native(crate::sys::tmq::Data),
    Ws(taos_ws::consumer::Data),
}

pub struct Offset(OffsetInner);
pub struct Meta(MetaInner);
pub struct Data(DataInner);

pub type MessageSet<Meta, Data> = taos_query::tmq::MessageSet<Meta, Data>;

pub struct TmqBuilder(TmqBuilderInner);
pub struct Consumer(ConsumerInner);

impl TBuilder for TmqBuilder {
    type Target = Consumer;

    type Error = super::Error;

    fn available_params() -> &'static [&'static str] {
        &[]
    }

    fn from_dsn<D: taos_query::IntoDsn>(dsn: D) -> Result<Self, Self::Error> {
        let dsn = dsn.into_dsn()?;
        // dbg!(&dsn);
        match (dsn.driver.as_str(), dsn.protocol.as_deref()) {
            ("ws" | "wss" | "http" | "https" | "taosws", _) => Ok(Self(TmqBuilderInner::Ws(
                taos_ws::consumer::TmqBuilder::from_dsn(dsn)?,
            ))),
            ("taos" | "tmq", None) => Ok(Self(TmqBuilderInner::Native(
                crate::sys::TmqBuilder::from_dsn(dsn)?,
            ))),
            ("taos" | "tmq", Some("ws" | "wss" | "http" | "https")) => Ok(Self(
                TmqBuilderInner::Ws(taos_ws::consumer::TmqBuilder::from_dsn(dsn)?),
            )),
            (driver, _) => Err(taos_query::DsnError::InvalidDriver(driver.to_string()).into()),
        }
    }

    fn client_version() -> &'static str {
        ""
    }

    fn ping(&self, conn: &mut Self::Target) -> Result<(), Self::Error> {
        match &self.0 {
            TmqBuilderInner::Native(b) => match &mut conn.0 {
                ConsumerInner::Native(taos) => Ok(b.ping(taos)?),
                _ => unreachable!(),
            },
            TmqBuilderInner::Ws(b) => match &mut conn.0 {
                ConsumerInner::Ws(taos) => Ok(b.ping(taos)?),
                _ => unreachable!(),
            },
        }
    }

    fn ready(&self) -> bool {
        match &self.0 {
            TmqBuilderInner::Native(b) => b.ready(),
            TmqBuilderInner::Ws(b) => b.ready(),
        }
    }

    fn build(&self) -> Result<Self::Target, Self::Error> {
        match &self.0 {
            TmqBuilderInner::Native(b) => Ok(Consumer(ConsumerInner::Native(b.build()?))),
            TmqBuilderInner::Ws(b) => Ok(Consumer(ConsumerInner::Ws(b.build()?))),
        }
    }

    fn server_version(&self) -> Result<&str, Self::Error> {
        todo!()
    }

    fn is_enterprise_edition(&self) -> bool {
        todo!()
    }
}

impl taos_query::tmq::IsOffset for Offset {
    fn database(&self) -> &str {
        match &self.0 {
            OffsetInner::Native(offset) => {
                <crate::sys::tmq::Offset as taos_query::tmq::IsOffset>::database(offset)
            }
            OffsetInner::Ws(offset) => {
                <taos_ws::consumer::Offset as taos_query::tmq::IsOffset>::database(offset)
            }
        }
    }

    fn topic(&self) -> &str {
        match &self.0 {
            OffsetInner::Native(offset) => {
                <crate::sys::tmq::Offset as taos_query::tmq::IsOffset>::topic(offset)
            }
            OffsetInner::Ws(offset) => {
                <taos_ws::consumer::Offset as taos_query::tmq::IsOffset>::topic(offset)
            }
        }
    }

    fn vgroup_id(&self) -> taos_query::tmq::VGroupId {
        match &self.0 {
            OffsetInner::Native(offset) => {
                <crate::sys::tmq::Offset as taos_query::tmq::IsOffset>::vgroup_id(offset)
            }
            OffsetInner::Ws(offset) => {
                <taos_ws::consumer::Offset as taos_query::tmq::IsOffset>::vgroup_id(offset)
            }
        }
    }
}

#[async_trait::async_trait]
impl taos_query::tmq::IsAsyncMeta for Meta {
    type Error = super::Error;

    async fn as_raw_meta(&self) -> Result<RawMeta, Self::Error> {
        match &self.0 {
            MetaInner::Native(data) => {
                <crate::sys::tmq::Meta as taos_query::tmq::IsAsyncMeta>::as_raw_meta(data)
                    .await
                    .map_err(Into::into)
            }
            MetaInner::Ws(data) => {
                <taos_ws::consumer::Meta as taos_query::tmq::IsAsyncMeta>::as_raw_meta(data)
                    .await
                    .map_err(Into::into)
            }
        }
    }

    async fn as_json_meta(&self) -> Result<taos_query::common::JsonMeta, Self::Error> {
        match &self.0 {
            MetaInner::Native(data) => {
                <crate::sys::tmq::Meta as taos_query::tmq::IsAsyncMeta>::as_json_meta(data)
                    .await
                    .map_err(Into::into)
            }
            MetaInner::Ws(data) => {
                <taos_ws::consumer::Meta as taos_query::tmq::IsAsyncMeta>::as_json_meta(data)
                    .await
                    .map_err(Into::into)
            }
        }
    }
}

#[async_trait::async_trait]
impl taos_query::tmq::IsAsyncData for Data {
    type Error = super::Error;

    async fn as_raw_data(&self) -> Result<taos_query::common::RawData, Self::Error> {
        match &self.0 {
            DataInner::Native(data) => {
                <crate::sys::tmq::Data as taos_query::tmq::IsAsyncData>::as_raw_data(data)
                    .await
                    .map_err(Into::into)
            }
            DataInner::Ws(data) => {
                <taos_ws::consumer::Data as taos_query::tmq::IsAsyncData>::as_raw_data(data)
                    .await
                    .map_err(Into::into)
            }
        }
    }

    async fn fetch_raw_block(&self) -> Result<Option<taos_query::RawBlock>, Self::Error> {
        match &self.0 {
            DataInner::Native(data) => {
                <crate::sys::tmq::Data as taos_query::tmq::IsAsyncData>::fetch_raw_block(data)
                    .await
                    .map_err(Into::into)
            }
            DataInner::Ws(data) => {
                <taos_ws::consumer::Data as taos_query::tmq::IsAsyncData>::fetch_raw_block(data)
                    .await
                    .map_err(Into::into)
            }
        }
    }
}

#[async_trait::async_trait]
impl AsAsyncConsumer for Consumer {
    type Error = super::Error;

    type Offset = Offset;

    type Meta = Meta;

    type Data = Data;

    fn default_timeout(&self) -> Timeout {
        match &self.0 {
            ConsumerInner::Native(c) => {
                <crate::sys::Consumer as AsAsyncConsumer>::default_timeout(c)
            }
            ConsumerInner::Ws(c) => {
                <taos_ws::consumer::Consumer as AsAsyncConsumer>::default_timeout(c)
            }
        }
    }

    async fn subscribe<T: Into<String>, I: IntoIterator<Item = T> + Send>(
        &mut self,
        topics: I,
    ) -> Result<(), Self::Error> {
        match &mut self.0 {
            ConsumerInner::Native(c) => {
                <crate::sys::Consumer as AsAsyncConsumer>::subscribe(c, topics)
                    .await
                    .map_err(Into::into)
            }
            ConsumerInner::Ws(c) => {
                <taos_ws::consumer::Consumer as AsAsyncConsumer>::subscribe(c, topics)
                    .await
                    .map_err(Into::into)
            }
        }
    }

    async fn recv_timeout(
        &self,
        timeout: Timeout,
    ) -> Result<Option<(Self::Offset, MessageSet<Self::Meta, Self::Data>)>, Self::Error> {
        match &self.0 {
            ConsumerInner::Native(c) => {
                <crate::sys::Consumer as AsAsyncConsumer>::recv_timeout(c, timeout)
                    .await
                    .map_err(Into::into)
                    .map(|msg| {
                        msg.map(|(offset, msg)| {
                            (
                                Offset(OffsetInner::Native(offset)),
                                match msg {
                                    MessageSet::Meta(meta) => {
                                        MessageSet::Meta(Meta(MetaInner::Native(meta)))
                                    }
                                    MessageSet::Data(data) => {
                                        MessageSet::Data(Data(DataInner::Native(data)))
                                    }
                                    MessageSet::MetaData(meta, data) => MessageSet::MetaData(
                                        Meta(MetaInner::Native(meta)),
                                        Data(DataInner::Native(data)),
                                    ),
                                },
                            )
                        })
                    })
            }
            ConsumerInner::Ws(c) => {
                <taos_ws::consumer::Consumer as AsAsyncConsumer>::recv_timeout(c, timeout)
                    .await
                    .map_err(Into::into)
                    .map(|msg| {
                        msg.map(|(offset, msg)| {
                            (
                                Offset(OffsetInner::Ws(offset)),
                                match msg {
                                    taos_query::tmq::MessageSet::Meta(meta) => {
                                        MessageSet::Meta(Meta(MetaInner::Ws(meta)))
                                    }
                                    taos_query::tmq::MessageSet::Data(data) => {
                                        MessageSet::Data(Data(DataInner::Ws(data)))
                                    }
                                    taos_query::tmq::MessageSet::MetaData(meta, data) => {
                                        MessageSet::MetaData(
                                            Meta(MetaInner::Ws(meta)),
                                            Data(DataInner::Ws(data)),
                                        )
                                    }
                                },
                            )
                        })
                    })
            }
        }
    }

    async fn commit(&self, offset: Self::Offset) -> Result<(), Self::Error> {
        match &self.0 {
            ConsumerInner::Native(c) => match offset.0 {
                OffsetInner::Native(offset) => {
                    <crate::sys::Consumer as AsAsyncConsumer>::commit(c, offset)
                        .await
                        .map_err(Into::into)
                }
                OffsetInner::Ws(_) => unreachable!(),
            },
            ConsumerInner::Ws(c) => match offset.0 {
                OffsetInner::Ws(offset) => {
                    <taos_ws::consumer::Consumer as AsAsyncConsumer>::commit(c, offset)
                        .await
                        .map_err(Into::into)
                }
                _ => unreachable!(),
            },
        }
    }
}

impl taos_query::tmq::SyncOnAsync for Consumer {}
impl taos_query::tmq::SyncOnAsync for Data {}
impl taos_query::tmq::SyncOnAsync for Meta {}

impl Iterator for Data {
    type Item = Result<RawBlock, super::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        use taos_query::prelude::IsAsyncData;
        block_in_place_or_global(self.fetch_raw_block()).transpose()
    }
}
// impl taos_query::tmq::AsConsumer for Consumer {}
#[cfg(test)]
mod tests {
    use std::{str::FromStr, time::Duration};

    use super::TmqBuilder;
    use crate::TaosBuilder;

    #[test]
    fn builder() -> anyhow::Result<()> {
        use taos_query::prelude::*;
        let mut dsn: Dsn = "taos://".parse()?;
        dsn.set("group.id", "group1");
        dsn.set("client.id", "test");
        dsn.set("auto.offset.reset", "earliest");

        let _tmq = TmqBuilder::from_dsn(dsn)?.build()?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_ws_tmq_meta() -> anyhow::Result<()> {
        // pretty_env_logger::formatted_timed_builder()
        //     .filter_level(log::LevelFilter::Debug)
        //     .init();
        use taos_query::prelude::*;
        let dsn = std::env::var("TEST_DSN").unwrap_or("taos://localhost:6030".to_string());
        let mut dsn = Dsn::from_str(&dsn)?;

        let taos = TaosBuilder::from_dsn(&dsn)?.build()?;
        taos.exec_many([
            "drop topic if exists ws_abc1",
            "drop database if exists ws_abc1",
            "create database ws_abc1",
            "create topic ws_abc1 with meta as database ws_abc1",
            "use ws_abc1",
            // kind 1: create super table using all types
            "create table stb1(ts timestamp, c1 bool, c2 tinyint, c3 smallint, c4 int, c5 bigint,\
            c6 timestamp, c7 float, c8 double, c9 varchar(10), c10 nchar(16),\
            c11 tinyint unsigned, c12 smallint unsigned, c13 int unsigned, c14 bigint unsigned)\
            tags(t1 json)",
            // kind 2: create child table with json tag
            "create table tb0 using stb1 tags('{\"name\":\"value\"}')",
            "create table tb1 using stb1 tags(NULL)",
            "insert into tb0 values(now, NULL, NULL, NULL, NULL, NULL,
            NULL, NULL, NULL, NULL, NULL,
            NULL, NULL, NULL, NULL)
            tb1 values(now, true, -2, -3, -4, -5, \
            '2022-02-02 02:02:02.222', -0.1, -0.12345678910, 'abc 和我', 'Unicode + 涛思',\
            254, 65534, 1, 1)",
            // kind 3: create super table with all types except json (especially for tags)
            "create table stb2(ts timestamp, c1 bool, c2 tinyint, c3 smallint, c4 int, c5 bigint,\
            c6 timestamp, c7 float, c8 double, c9 varchar(10), c10 nchar(10),\
            c11 tinyint unsigned, c12 smallint unsigned, c13 int unsigned, c14 bigint unsigned)\
            tags(t1 bool, t2 tinyint, t3 smallint, t4 int, t5 bigint,\
            t6 timestamp, t7 float, t8 double, t9 varchar(10), t10 nchar(16),\
            t11 tinyint unsigned, t12 smallint unsigned, t13 int unsigned, t14 bigint unsigned)",
            // kind 4: create child table with all types except json
            "create table tb2 using stb2 tags(true, -2, -3, -4, -5, \
            '2022-02-02 02:02:02.222', -0.1, -0.12345678910, 'abc 和我', 'Unicode + 涛思',\
            254, 65534, 1, 1)",
            "create table tb3 using stb2 tags( NULL, NULL, NULL, NULL, NULL,
            NULL, NULL, NULL, NULL, NULL,
            NULL, NULL, NULL, NULL)",
            // kind 5: create common table
            "create table `table` (ts timestamp, v int)",
            // kind 6: column in super table
            "alter table stb1 add column new1 bool",
            "alter table stb1 add column new2 tinyint",
            "alter table stb1 add column new10 nchar(16)",
            "alter table stb1 modify column new10 nchar(32)",
            "alter table stb1 drop column new10",
            "alter table stb1 drop column new2",
            "alter table stb1 drop column new1",
            // kind 7: add tag in super table
            "alter table `stb2` add tag new1 bool",
            "alter table `stb2` rename tag new1 new1_new",
            "alter table `stb2` modify tag t10 nchar(32)",
            "alter table `stb2` drop tag new1_new",
            // kind 8: column in common table
            "alter table `table` add column new1 bool",
            "alter table `table` add column new2 tinyint",
            "alter table `table` add column new10 nchar(16)",
            "alter table `table` modify column new10 nchar(32)",
            "alter table `table` rename column new10 new10_new",
            "alter table `table` drop column new10_new",
            "alter table `table` drop column new2",
            "alter table `table` drop column new1",
            // kind 9: drop normal table
            "drop table `table`",
            // kind 10: drop child table
            "drop table `tb2`, `tb1`",
            // kind 11: drop super table
            "drop table `stb2`",
            "drop table `stb1`",
        ])
        .await?;

        taos.exec_many([
            "drop database if exists db2",
            "create database if not exists db2",
            "use db2",
        ])
        .await?;

        dsn.params.insert("group.id".to_string(), "abc".to_string());
        let builder = TmqBuilder::from_dsn(&dsn)?;
        let mut consumer = builder.build()?;
        consumer.subscribe(["ws_abc1"]).await?;

        {
            let mut stream = consumer.stream_with_timeout(Timeout::from_secs(1));

            while let Some((offset, message)) = stream.try_next().await? {
                // Offset contains information for topic name, database name and vgroup id,
                //  similar to kafka topic/partition/offset.
                let _ = offset.topic();
                let _ = offset.database();
                let _ = offset.vgroup_id();

                // Different to kafka message, TDengine consumer would consume two kind of messages.
                //
                // 1. meta
                // 2. data
                match message {
                    MessageSet::Meta(meta) => {
                        let raw = meta.as_raw_meta().await?;
                        taos.write_raw_meta(&raw).await?;

                        // meta data can be write to an database seamlessly by raw or json (to sql).
                        let json = meta.as_json_meta().await?;
                        let sql = json.to_string();
                        if let Err(err) = taos.exec(sql).await {
                            println!("maybe error: {}", err);
                        }
                    }
                    MessageSet::Data(data) => {
                        // data message may have more than one data block for various tables.
                        while let Some(data) = data.fetch_raw_block().await? {
                            dbg!(data.table_name());
                            dbg!(data);
                        }
                    }
                    MessageSet::MetaData(meta, data) => {
                        let raw = meta.as_raw_meta().await?;
                        taos.write_raw_meta(&raw).await?;

                        // meta data can be write to an database seamlessly by raw or json (to sql).
                        let json = meta.as_json_meta().await?;
                        let sql = json.to_string();
                        if let Err(err) = taos.exec(sql).await {
                            println!("maybe error: {}", err);
                        }
                        // data message may have more than one data block for various tables.
                        while let Some(data) = data.fetch_raw_block().await? {
                            dbg!(data.table_name());
                            dbg!(data);
                        }
                    }
                }
                consumer.commit(offset).await?;
            }
        }
        consumer.unsubscribe().await;

        tokio::time::sleep(Duration::from_secs(2)).await;

        taos.exec_many([
            "drop database db2",
            "drop topic ws_abc1",
            "drop database ws_abc1",
        ])
        .await?;
        Ok(())
    }
}
