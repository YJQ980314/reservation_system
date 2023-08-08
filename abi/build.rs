use std::process::Command;

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .type_attribute("reservation.ReservationStatus", "#[derive(sqlx::Type)]")
        // 该方法用于为指定的 Rust 数据类型添加属性或注解。在这个例子中，它添加了一个 #[derive(sqlx::Type)] 属性
        // 给名为 reservation.ReservationStatus 的数据类型。这个属性的目的可能是为了在 sqlx 数据库库中让数据类型
        // 自动实现 sqlx::Type trait，以便与数据库的映射进行交互。sqlx::Type 是 sqlx 数据库库提供的一个 trait，
        // 用于表示 Rust 数据类型与数据库中的数据类型之间的映射关系。它定义了一组方法，允许你在 Rust 代码和数据库之间进行数据类型的转换和映射。
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();

    // use std::fs;
    // fs::remove_file("src/pb/google.protobuf.rs").unwrap();

    Command::new("cargo").args(["fmt"]).output().unwrap();

    println!("cargo:rerun-if-changed=protos/reservation.proto")
}
