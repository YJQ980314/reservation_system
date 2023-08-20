use std::process::Command;
use tonic_build::Builder;

fn main() {
    // tonic_build::configure()
    //     .out_dir("src/pb")
    //     .type_attribute("reservation.ReservationStatus", "#[derive(sqlx::Type)]")
    //     .type_attribute(
    //         "reservation.ReservationQuery",
    //         "#[derive(derive_builder::Builder)]",
    //     )
    //     .field_attribute(
    //         "reservation.ReservationQuery.start",
    //         "#[builder(setter(into, strip_option))]",
    //     )
    //     .field_attribute(
    //         "reservation.ReservationQuery.end",
    //         "#[builder(setter(into, strip_option))]",
    //     )
    //     // 该方法用于为指定的 Rust 数据类型添加属性或注解。在这个例子中，它添加了一个 #[derive(sqlx::Type)] 属性
    //     // 给名为 reservation.ReservationStatus 的数据类型。这个属性的目的可能是为了在 sqlx 数据库库中让数据类型
    //     // 自动实现 sqlx::Type trait，以便与数据库的映射进行交互。sqlx::Type 是 sqlx 数据库库提供的一个 trait，
    //     // 用于表示 Rust 数据类型与数据库中的数据类型之间的映射关系。它定义了一组方法，允许你在 Rust 代码和数据库之间进行数据类型的转换和映射。
    //     .compile(&["protos/reservation.proto"], &["protos"])
    //     .unwrap();

    tonic_build::configure()
        .out_dir("src/pb")
        .with_sqlx_type(&["reservation.ReservationStatus"])
        .with_derive_builder(&[
            "reservation.ReservationQuery",
            "reservation.ReservationFilter",
        ])
        .with_derive_builder_into(
            "reservation.ReservationQuery",
            &[
                "resource_id",
                "user_id",
                "status",
                "page",
                "page_size",
                "desc",
            ],
        )
        .with_derive_builder_into(
            "reservation.ReservationFilter",
            &["resource_id", "user_id", "status", "desc"],
        )
        .with_derive_builder_option("reservation.ReservationFilter", &["cursor"])
        .with_derive_builder_option("reservation.ReservationQuery", &["start", "end"])
        .with_type_attributes(
            &[
                "reservation.ReservationFilter",
                "reservation.ReservationQuery",
            ],
            &[r#"#[builder(build_fn(name = "private_build"))]"#],
        )
        .with_field_attributes(
            &["page_size"],
            &["#[builder(setter(into), default = \"10\")]"],
        )
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();

    // use std::fs;
    // fs::remove_file("src/pb/google.protobuf.rs").unwrap();

    Command::new("cargo").args(["fmt"]).output().unwrap();

    println!("cargo:rerun-if-changed=protos/reservation.proto")
}

trait BuildExt {
    fn with_sqlx_type(self, paths: &[&str]) -> Self;
    fn with_derive_builder(self, paths: &[&str]) -> Self;
    fn with_derive_builder_into(self, path: &str, fields: &[&str]) -> Self;
    fn with_derive_builder_option(self, path: &str, fields: &[&str]) -> Self;
    fn with_field_attributes(self, paths: &[&str], attributes: &[&str]) -> Self;
    fn with_type_attributes(self, paths: &[&str], attributes: &[&str]) -> Self;
}

impl BuildExt for Builder {
    fn with_sqlx_type(self, paths: &[&str]) -> Self {
        paths.iter().fold(self, |acc, path| {
            acc.type_attribute(path, "#[derive(sqlx::Type)]")
        })
    }

    fn with_derive_builder(self, paths: &[&str]) -> Self {
        paths.iter().fold(self, |acc, path| {
            acc.type_attribute(path, "#[derive(derive_builder::Builder)]")
        })
    }

    fn with_derive_builder_into(self, path: &str, fields: &[&str]) -> Self {
        fields.iter().fold(self, |acc, field| {
            acc.field_attribute(
                format!("{}.{}", path, field),
                "#[builder(setter(into), default)]",
            )
        })
    }

    fn with_derive_builder_option(self, path: &str, fields: &[&str]) -> Self {
        fields.iter().fold(self, |acc, field| {
            acc.field_attribute(
                format!("{}.{}", path, field),
                "#[builder(setter(into, strip_option), default)]",
            )
        })
    }
    fn with_field_attributes(self, paths: &[&str], attributes: &[&str]) -> Self {
        let attr = attributes.join("\n");
        paths.iter().fold(self, |builder, ty| {
            builder.field_attribute(ty, attr.as_str())
        })
    }

    fn with_type_attributes(self, paths: &[&str], attributes: &[&str]) -> Self {
        let attr = attributes.join("\n");

        paths.iter().fold(self, |builder, ty| {
            builder.type_attribute(ty, attr.as_str())
        })
    }
}
