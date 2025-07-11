use std::sync::Arc;

use datafusion::catalog_common::TableReference;
use exon::ExonSession;
use log::{debug, info};
use sequila_core::session_context::{Algorithm, SequilaConfig};
use tokio::runtime::Runtime;

use crate::context::set_option_internal;
use crate::option::{FilterOp, RangeOp, RangeOptions, QCOptions, QCOp};
use crate::query::{nearest_query, overlap_query, mean_quality_query, mean_quality_histogram_query};
use crate::udtf::CountOverlapsProvider;
use crate::utils::default_cols_to_string;
use crate::DEFAULT_COLUMN_NAMES;

pub(crate) struct QueryParams {
    pub sign: String,
    pub suffixes: (String, String),
    pub columns_1: Vec<String>,
    pub columns_2: Vec<String>,
    pub other_columns_1: Vec<String>,
    pub other_columns_2: Vec<String>,
    pub left_table: String,
    pub right_table: String,
}
pub(crate) fn do_range_operation(
    ctx: &ExonSession,
    rt: &Runtime,
    range_options: RangeOptions,
    left_table: String,
    right_table: String,
) -> datafusion::dataframe::DataFrame {
    // defaults
    match &range_options.overlap_alg {
        Some(alg) if alg == "coitreesnearest" => {
            panic!("CoitreesNearest is an internal algorithm for nearest operation. Can't be set explicitly.");
        },
        Some(alg) => {
            set_option_internal(ctx, "sequila.interval_join_algorithm", alg);
        },
        _ => {
            set_option_internal(
                ctx,
                "sequila.interval_join_algorithm",
                &Algorithm::Coitrees.to_string(),
            );
        },
    }
    let streaming = range_options.streaming.unwrap_or(false);
    if streaming {
        info!("Running in streaming mode...");
    }
    info!(
        "Running {} operation with algorithm {} and {} thread(s)...",
        range_options.range_op,
        ctx.session
            .state()
            .config()
            .options()
            .extensions
            .get::<SequilaConfig>()
            .unwrap()
            .interval_join_algorithm,
        ctx.session
            .state()
            .config()
            .options()
            .execution
            .target_partitions
    );
    match range_options.range_op {
        RangeOp::Overlap => rt.block_on(do_overlap(ctx, range_options, left_table, right_table)),
        RangeOp::Nearest => {
            set_option_internal(ctx, "sequila.interval_join_algorithm", "coitreesnearest");
            rt.block_on(do_nearest(ctx, range_options, left_table, right_table))
        },
        RangeOp::CountOverlapsNaive => rt.block_on(do_count_overlaps_coverage_naive(
            ctx,
            range_options,
            left_table,
            right_table,
            false,
        )),
        RangeOp::Coverage => rt.block_on(do_count_overlaps_coverage_naive(
            ctx,
            range_options,
            left_table,
            right_table,
            true,
        )),

        _ => panic!("Unsupported operation"),
    }
}

async fn do_nearest(
    ctx: &ExonSession,
    range_opts: RangeOptions,
    left_table: String,
    right_table: String,
) -> datafusion::dataframe::DataFrame {
    let query = prepare_query(nearest_query, range_opts, ctx, left_table, right_table)
        .await
        .to_string();
    debug!("Query: {}", query);
    ctx.sql(&query).await.unwrap()
}

async fn do_overlap(
    ctx: &ExonSession,
    range_opts: RangeOptions,
    left_table: String,
    right_table: String,
) -> datafusion::dataframe::DataFrame {
    let query = prepare_query(overlap_query, range_opts, ctx, left_table, right_table)
        .await
        .to_string();
    debug!("Query: {}", query);
    debug!(
        "{}",
        ctx.session
            .state()
            .config()
            .options()
            .execution
            .target_partitions
    );
    ctx.sql(&query).await.unwrap()
}

async fn do_count_overlaps_coverage_naive(
    ctx: &ExonSession,
    range_opts: RangeOptions,
    left_table: String,
    right_table: String,
    coverage: bool,
) -> datafusion::dataframe::DataFrame {
    let columns_1 = range_opts.columns_1.unwrap();
    let columns_2 = range_opts.columns_2.unwrap();
    let session = &ctx.session;
    let right_table_ref = TableReference::from(right_table.clone());
    let right_schema = session
        .table(right_table_ref.clone())
        .await
        .unwrap()
        .schema()
        .as_arrow()
        .clone();
    let count_overlaps_provider = CountOverlapsProvider::new(
        Arc::new(session.clone()),
        left_table,
        right_table,
        right_schema,
        columns_1,
        columns_2,
        range_opts.filter_op.unwrap(),
        coverage,
    );
    let table_name = "count_overlaps_coverage".to_string();
    session.deregister_table(table_name.clone()).unwrap();
    session
        .register_table(table_name.clone(), Arc::new(count_overlaps_provider))
        .unwrap();
    let query = format!("SELECT * FROM {}", table_name);
    debug!("Query: {}", query);
    ctx.sql(&query).await.unwrap()
}


use arrow::array::{LargeStringArray, ArrayRef, Int32Builder};
use arrow_array::Array;
use arrow::datatypes::{Field, Schema,DataType};
use arrow::record_batch::RecordBatch;
use rayon::prelude::*;

pub fn compute_mean_c_parallel(
    batches: Vec<RecordBatch>,
    num_threads: Option<usize>,
) -> Vec<RecordBatch> {
    if let Some(n) = num_threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .ok(); // Ignorujemy błąd, bo build_global można wykonać tylko raz
    }

    batches
        .into_par_iter()
        .map(|batch| {
            let quality_col = batch
                .column_by_name("quality_scores")
                .expect("Column 'quality_scores' not found in RecordBatch")
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .expect("Column 'quality_scores' is not a LargeStringArray");

            let mut means = Int32Builder::with_capacity(quality_col.len());
            for i in 0..quality_col.len() {
                if quality_col.is_null(i) {
                    means.append_null();
                } else {
                    let qstr = quality_col.value(i);
                    let mean = qstr
                        .as_bytes()
                        .iter()
                        .map(|b| (*b as i32 - 33))
                        .sum::<i32>()
                        / qstr.len() as i32;
                    means.append_value(mean);
                }
            }

            let mean_array: ArrayRef = Arc::new(means.finish());

            let mut cols = batch.columns().to_vec();
            cols.push(mean_array);

            let mut fields: Vec<Field> = batch
                .schema()
                .fields()
                .iter()
                .map(|f| f.as_ref().clone())
                .collect();
            fields.push(Field::new("mean_c", DataType::Int32, true));

            let schema = Arc::new(Schema::new(fields));
            RecordBatch::try_new(schema, cols).unwrap()
        })
        .collect()
}


// wersja Int32
pub fn compute_mean_c(batches: Vec<RecordBatch>) -> Vec<RecordBatch> {
    batches
        .into_iter()
        .map(|batch| {
            let quality_col = batch
                .column_by_name("quality_scores")
                .expect("Column 'quality_scores' not found in RecordBatch")
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .expect("Column 'quality_scores' is not a LargeStringArray");

            let mut means = Int32Builder::with_capacity(quality_col.len());
            for i in 0..quality_col.len() {
                if quality_col.is_null(i) {
                    means.append_null();
                } else {
                    let qstr = quality_col.value(i);
                    let mean = qstr
                        .as_bytes()
                        .iter()
                        .map(|b| (*b as i32 - 33))
                        .sum::<i32>()
                        / qstr.len() as i32;
                    means.append_value(mean);
                }
            }

            let mean_array: ArrayRef = Arc::new(means.finish());

            let mut cols = batch.columns().to_vec();
            cols.push(mean_array);

            let mut fields: Vec<Field> = batch
                .schema()
                .fields()
                .iter()
                .map(|f| f.as_ref().clone())
                .collect();
            fields.push(Field::new("mean_c", DataType::Int32, true));

            let schema = Arc::new(Schema::new(fields));
            RecordBatch::try_new(schema, cols).unwrap()
        })
        .collect()
}

pub fn do_qc_operation(
    ctx: &ExonSession,
    rt: &Runtime,
    qc_options: QCOptions,
    table_name: String,
) -> datafusion::dataframe::DataFrame {

    info!(
        "Running operation with {} thread(s)...",
        ctx.session
            .state()
            .config()
            .options()
            .execution
            .target_partitions
    );


    match qc_options.qc_op {
        QCOp::MeanQuality => rt.block_on(do_mean_quality(ctx, qc_options, table_name)),
        QCOp::MeanQualityHistogram => rt.block_on(do_mean_quality_histogram(ctx, qc_options, table_name)),
        _ => panic!("Unsupported operation"),
    }
}

pub async fn do_mean_quality_histogram(
    ctx: &ExonSession,
    qc_options: QCOptions,
    table: String,
)  -> datafusion::dataframe::DataFrame {
    let bin_size: u32 = 1 as u32; // szerokość bina histogramu, spakować w qc_options
    // qc_options.quality_col to nazwa kolumny dodanej w rust, która zawiera uśrednioną jakość dla każdego odczytu
    // table to nazwa zarejestrowanej tabeli
    let mean_qual_col = String::from("mean_c");
    let query = mean_quality_histogram_query(table, bin_size,  mean_qual_col)
        .to_string();
    debug!("Query: {}", query);
    debug!(
        "{}",
        ctx.session
            .state()
            .config()
            .options()
            .execution
            .target_partitions
    );
    // println!("Query : {}", query);
    ctx.sql(&query).await.unwrap() // zwróci datafusion::dataframe::DataFrame z histogramem
}

pub async fn do_mean_quality(
    ctx: &ExonSession,
    qc_options: QCOptions,
    table: String,
)  -> datafusion::dataframe::DataFrame {
    let bin_size: u32 = 1 as u32; // szerokość bina histogramu, spakować w qc_options
    // qc_options.quality_col to nazwa kolumny dodanej w rust, która zawiera uśrednioną jakość dla każdego odczytu
    // table to nazwa zarejestrowanej tabeli
    let mean_qual_col = String::from("mean_c");
    let query = mean_quality_query(table,  mean_qual_col)
        .to_string();
    debug!("Query: {}", query);
    debug!(
        "{}",
        ctx.session
            .state()
            .config()
            .options()
            .execution
            .target_partitions
    );
    // println!("Query : {}", query);
    ctx.sql(&query).await.unwrap() // zwróci datafusion::dataframe::DataFrame z histogramem
}


async fn get_non_join_columns(
    table_name: String,
    join_columns: Vec<String>,
    ctx: &ExonSession,
) -> Vec<String> {
    let table_ref = TableReference::from(table_name);
    let table = ctx.session.table(table_ref).await.unwrap();
    table
        .schema()
        .fields()
        .iter()
        .map(|f| f.name().to_string())
        .filter(|f| !join_columns.contains(f))
        .collect::<Vec<String>>()
}

pub(crate) fn format_non_join_tables(
    columns: Vec<String>,
    table_alias: String,
    suffix: String,
) -> String {
    if columns.is_empty() {
        return "".to_string();
    }
    columns
        .iter()
        .map(|c| format!("{}.{} as {}{}", table_alias, c, c, suffix))
        .collect::<Vec<String>>()
        .join(", ")
}

pub(crate) async fn prepare_query(
    query: fn(QueryParams) -> String,
    range_opts: RangeOptions,
    ctx: &ExonSession,
    left_table: String,
    right_table: String,
) -> String {
    let sign = match range_opts.filter_op.unwrap() {
        FilterOp::Weak => "=".to_string(),
        _ => "".to_string(),
    };
    let suffixes = match range_opts.suffixes {
        Some((s1, s2)) => (s1, s2),
        _ => ("_1".to_string(), "_2".to_string()),
    };
    let columns_1 = match range_opts.columns_1 {
        Some(cols) => cols,
        _ => default_cols_to_string(&DEFAULT_COLUMN_NAMES),
    };
    let columns_2 = match range_opts.columns_2 {
        Some(cols) => cols,
        _ => default_cols_to_string(&DEFAULT_COLUMN_NAMES),
    };

    let left_table_columns =
        get_non_join_columns(left_table.to_string(), columns_1.clone(), ctx).await;
    let right_table_columns =
        get_non_join_columns(right_table.to_string(), columns_2.clone(), ctx).await;

    let query_params = QueryParams {
        sign,
        suffixes,
        columns_1,
        columns_2,
        other_columns_1: left_table_columns,
        other_columns_2: right_table_columns,
        left_table,
        right_table,
    };

    query(query_params)
}
