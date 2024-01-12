pub mod imserver;
pub mod imclient;
pub mod exserver;
// pub mod slowloris; 


// 版权归亚马逊公司及其关联公司所有。保留所有权利。
// SPDX-License-Identifier: Apache-2.0

/// 用于Slowloris风格的拒绝服务攻击的示例缓解方法。有关此攻击的详细信息，
/// 请参阅[QUIC§21.6](https://www.rfc-editor.org/rfc/rfc9000.html#name-slowloris-attacks)。
///
/// 本示例中使用的连接监控器还可用于缓解[QUIC§21.9](https://www.rfc-editor.org/rfc/rfc9000.html#name-peer-denial-of-service)中描述的更一般的对等拒绝服务攻击。
pub mod slowloris {
    use s2n_quic::provider::{
        event,
        event::{events, supervisor, ConnectionMeta, Timestamp},
    };
    use std::time::Duration;

    /// 在低吞吐量连接关闭之前，同时活动的最大连接数。
    const CONNECTION_COUNT_THRESHOLD: usize = 1000;
    /// 连接必须维持的最小吞吐量，以字节每秒计
    const MIN_THROUGHPUT: usize = 500;

    /// 定义包含要跟踪的每个连接的任何连接状态的Connection Context。
    /// 对于此示例，我们需要跟踪传输的字节数和传输字节数上次总计的时间。
    #[derive(Debug, Clone)]
    pub struct MyConnectionContext {
        transferred_bytes: usize,
        last_update: Timestamp,
    }

    /// 定义一个包含要在所有连接上跟踪的任何状态的结构。
    /// 对于此示例，没有额外的状态需要跟踪，因此该结构为空。
    #[derive(Default)]
    pub struct MyConnectionSupervisor;

    /// 为结构实现`event::Subscriber`特性。必须实现`create_connection_context`方法，
    /// 以初始化每个连接的Connection Context。其他方法可以根据需要实现。
    impl event::Subscriber for MyConnectionSupervisor {
        type ConnectionContext = MyConnectionContext;

        /// 初始化Connection Context，该上下文传递给`supervisor_timeout`和
        /// `on_supervisor_timeout`方法，以及每个与连接相关的事件。
        fn create_connection_context(
            &mut self,
            meta: &events::ConnectionMeta,
            _info: &events::ConnectionInfo,
        ) -> Self::ConnectionContext {
            MyConnectionContext {
                transferred_bytes: 0,
                last_update: meta.timestamp,
            }
        }

        /// 实现`supervisor_timeout`以定义`on_supervisor_timeout`将在何时调用。
        /// 对于此示例，使用常量1秒，但此值可以根据连接随时间变化或基于连接调整。
        fn supervisor_timeout(
            &mut self,
            _conn_context: &mut Self::ConnectionContext,
            _meta: &ConnectionMeta,
            _context: &supervisor::Context,
        ) -> Option<Duration> {
            Some(Duration::from_secs(1))
        }

        /// 实现`on_supervisor_timeout`以定义`supervisor_timeout`过期时应采取的连接操作。
        /// 对于此示例，如果打开的连接数大于`CONNECTION_COUNT_THRESHOLD`且连接的吞吐量
        /// 自上次`supervisor_timeout`以来下降到`MIN_THROUGHPUT`以下，则立即关闭连接
        /// (`supervisor::Outcome::ImmediateClose`)。
        fn on_supervisor_timeout(
            &mut self,
            conn_context: &mut Self::ConnectionContext,
            meta: &ConnectionMeta,
            context: &supervisor::Context,
        ) -> supervisor::Outcome {
            if !context.is_handshaking && context.connection_count > CONNECTION_COUNT_THRESHOLD {
                let elapsed_time = meta.timestamp.duration_since_start()
                    - conn_context.last_update.duration_since_start();

                // 计算吞吐量，单位为每秒字节数
                let throughput =
                    (conn_context.transferred_bytes as f32 / elapsed_time.as_secs_f32()) as usize;

                if throughput < MIN_THROUGHPUT {
                    // 立即关闭连接，而不通知对等方
                    return supervisor::Outcome::ImmediateClose {
                        reason: "Connection throughput was below MIN_THROUGHPUT",
                    };
                }
            }

            // 更新`last_update`时间戳并重置传输字节
            conn_context.last_update = meta.timestamp;
            conn_context.transferred_bytes = 0;

            // 允许连接继续
            supervisor::Outcome::Continue
        }

        /// 实现`on_tx_stream_progress`，在传出流上取得进展时通知每次。
        fn on_tx_stream_progress(
            &mut self,
            context: &mut Self::ConnectionContext,
            _meta: &events::ConnectionMeta,
            event: &events::TxStreamProgress,
        ) {
            context.transferred_bytes += event.bytes;
        }

        /// 实现`on_rx_stream_progress`，在传入流上取得进展时通知每次。
        fn on_rx_stream_progress(
            &mut self,
            context: &mut Self::ConnectionContext,
            _meta: &events::ConnectionMeta,
            event: &events::RxStreamProgress,
        ) {
            context.transferred_bytes += event.bytes;
        }
    }
}


// // Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// // SPDX-License-Identifier: Apache-2.0

// /// Example mitigation for Slowloris-style Denial of Service attacks. For details on this attack,
// /// see [QUIC§21.6](https://www.rfc-editor.org/rfc/rfc9000.html#name-slowloris-attacks).
// ///
// /// The Connection Supervisor used in this example may also be used to mitigate the more general
// /// Peer Denial of Service attack described in [QUIC§21.9](https://www.rfc-editor.org/rfc/rfc9000.html#name-peer-denial-of-service).
// pub mod slowloris {
//     use s2n_quic::provider::{
//         event,
//         event::{events, supervisor, ConnectionMeta, Timestamp},
//     };
//     use std::time::Duration;

//     /// The maximum number of connections that may be active concurrently before
//     /// low throughput connections are closed.
//     const CONNECTION_COUNT_THRESHOLD: usize = 1000;
//     /// The minimum throughput a connection must sustain, in bytes per second
//     const MIN_THROUGHPUT: usize = 500;

//     /// Define a Connection Context containing any per-connection state you wish to track.
//     /// For this example, we need to track the number of bytes transferred and the last
//     /// time the transferred byte count was totalled.
//     #[derive(Debug, Clone)]
//     pub struct MyConnectionContext {
//         transferred_bytes: usize,
//         last_update: Timestamp,
//     }

//     /// Define a struct containing any state across all connections you wish to track.
//     /// For this example, there is no additional state we need to track so the struct is empty.
//     #[derive(Default)]
//     pub struct MyConnectionSupervisor;

//     /// Implement the `event::Subscriber` trait for your struct. The `create_connection_context`
//     /// method must be implemented to initialize the Connection Context for each connection.
//     /// Other methods may be implemented as needed.
//     impl event::Subscriber for MyConnectionSupervisor {
//         type ConnectionContext = MyConnectionContext;

//         /// Initialize the Connection Context that is passed to the `supervisor_timeout` and
//         /// `on_supervisor_timeout` methods, as well as each connection-related event.
//         fn create_connection_context(
//             &mut self,
//             meta: &events::ConnectionMeta,
//             _info: &events::ConnectionInfo,
//         ) -> Self::ConnectionContext {
//             MyConnectionContext {
//                 transferred_bytes: 0,
//                 last_update: meta.timestamp,
//             }
//         }

//         /// Implement `supervisor_timeout` to define the period at which `on_supervisor_timeout` will
//         /// be invoked. For this example, a constant of 1 second is used, but this value can be
//         /// varied over time or based on the connection.
//         fn supervisor_timeout(
//             &mut self,
//             _conn_context: &mut Self::ConnectionContext,
//             _meta: &ConnectionMeta,
//             _context: &supervisor::Context,
//         ) -> Option<Duration> {
//             Some(Duration::from_secs(1))
//         }

//         /// Implement `on_supervisor_timeout` to define what action should be taken on the connection
//         /// when the `supervisor_timeout` expires. For this example, the connection will be closed
//         /// immediately (`supervisor::Outcome::ImmediateClose`) if the number of open connections
//         /// is greater than `CONNECTION_COUNT_THRESHOLD` and the the throughput of the connection
//         /// since the last `supervisor_timeout` has dropped below `MIN_THROUGHPUT`.
//         fn on_supervisor_timeout(
//             &mut self,
//             conn_context: &mut Self::ConnectionContext,
//             meta: &ConnectionMeta,
//             context: &supervisor::Context,
//         ) -> supervisor::Outcome {
//             if !context.is_handshaking && context.connection_count > CONNECTION_COUNT_THRESHOLD {
//                 let elapsed_time = meta.timestamp.duration_since_start()
//                     - conn_context.last_update.duration_since_start();

//                 // Calculate throughput as bytes per second
//                 let throughput =
//                     (conn_context.transferred_bytes as f32 / elapsed_time.as_secs_f32()) as usize;

//                 if throughput < MIN_THROUGHPUT {
//                     // Close the connection immediately without notifying the peer
//                     return supervisor::Outcome::ImmediateClose {
//                         reason: "Connection throughput was below MIN_THROUGHPUT",
//                     };
//                 }
//             }

//             // Update the `last_update` timestamp and reset transferred bytes
//             conn_context.last_update = meta.timestamp;
//             conn_context.transferred_bytes = 0;

//             // Allow the connection to continue
//             supervisor::Outcome::Continue
//         }

//         /// Implement `on_tx_stream_progress` to be notified every time forward progress is made
//         /// on an outgoing stream.
//         fn on_tx_stream_progress(
//             &mut self,
//             context: &mut Self::ConnectionContext,
//             _meta: &events::ConnectionMeta,
//             event: &events::TxStreamProgress,
//         ) {
//             context.transferred_bytes += event.bytes;
//         }

//         /// Implement `on_rx_progress` to be notified every time forward progress is made on an
//         /// incoming stream.
//         fn on_rx_stream_progress(
//             &mut self,
//             context: &mut Self::ConnectionContext,
//             _meta: &events::ConnectionMeta,
//             event: &events::RxStreamProgress,
//         ) {
//             context.transferred_bytes += event.bytes;
//         }
//     }
// }
