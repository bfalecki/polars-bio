use std::fmt;

use datafusion_bio_format_core::object_storage::{CompressionType, ObjectStorageOptions};
use pyo3::{pyclass, pymethods};

#[pyclass(name = "RangeOptions")]
#[derive(Clone, Debug)]
pub struct RangeOptions {
    #[pyo3(get, set)]
    pub range_op: RangeOp,
    #[pyo3(get, set)]
    pub filter_op: Option<FilterOp>,
    #[pyo3(get, set)]
    pub suffixes: Option<(String, String)>,
    #[pyo3(get, set)]
    pub columns_1: Option<Vec<String>>,
    #[pyo3(get, set)]
    pub columns_2: Option<Vec<String>>,
    #[pyo3(get, set)]
    on_cols: Option<Vec<String>>,
    #[pyo3(get, set)]
    pub overlap_alg: Option<String>,
    #[pyo3(get, set)]
    pub streaming: Option<bool>,
}



#[pymethods]
impl RangeOptions {
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(signature = (range_op, filter_op=None, suffixes=None, columns_1=None, columns_2=None, on_cols=None, overlap_alg=None, streaming=None))]
    pub fn new(
        range_op: RangeOp,
        filter_op: Option<FilterOp>,
        suffixes: Option<(String, String)>,
        columns_1: Option<Vec<String>>,
        columns_2: Option<Vec<String>>,
        on_cols: Option<Vec<String>>,
        overlap_alg: Option<String>,
        streaming: Option<bool>,
    ) -> Self {
        RangeOptions {
            range_op,
            filter_op,
            suffixes,
            columns_1,
            columns_2,
            on_cols,
            overlap_alg,
            streaming,
        }
    }
}
impl std::fmt::Display for RangeOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "streaming {}", self.streaming.unwrap_or(false))
    }
}


#[pyclass(name = "QCOptions")]
#[derive(Clone, Debug)]
pub struct QCOptions {
    #[pyo3(get, set)]
    pub qc_op: QCOp,  // e.g. MeanQuality, GCContent, ...
    #[pyo3(get, set)]
    pub quality_col: Option<String>,
    #[pyo3(get, set)]
    pub output_col: Option<String>,
    #[pyo3(get, set)]
    pub ascii_offset: Option<u8>,
    #[pyo3(get, set)]
    pub streaming: Option<bool>,
}

#[pymethods]
impl QCOptions {
    #[new]
    #[pyo3(signature = (
        qc_op,
        quality_col = None,
        output_col = None,
        ascii_offset = None,
        streaming = None
    ))]
    pub fn new(
        qc_op: QCOp,
        quality_col: Option<String>,
        output_col: Option<String>,
        ascii_offset: Option<u8>,
        streaming: Option<bool>,
    ) -> Self {
        QCOptions {
            qc_op,
            quality_col,
            output_col,
            ascii_offset,
            streaming,
        }
    }
}

impl std::fmt::Display for QCOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "qc_op: {}, streaming: {}",
            // self.qc_op,
            "Display not possible",
            self.streaming.unwrap_or(false)
        )
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub enum QCOp {
    MeanQuality,
    MeanQualityHistogram,
    // GCContent, (in future)
    // other
}



#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq, Debug)]
pub enum FilterOp {
    Weak = 0,
    Strict = 1,
}

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq, Debug)]
pub enum RangeOp {
    Overlap = 0,
    Complement = 1,
    Cluster = 2,
    Nearest = 3,
    Coverage = 4,
    CountOverlapsNaive = 6,
}

impl fmt::Display for RangeOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RangeOp::Overlap => write!(f, "Overlap"),
            RangeOp::Nearest => write!(f, "Nearest"),
            RangeOp::Complement => write!(f, "Complement"),
            RangeOp::Cluster => write!(f, "Cluster"),
            RangeOp::Coverage => write!(f, "Coverage"),
            RangeOp::CountOverlapsNaive => write!(f, "Count overlaps naive"),
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq, Debug)]
pub enum InputFormat {
    Parquet,
    Csv,
    Bam,
    IndexedBam,
    Cram,
    Vcf,
    IndexedVcf,
    Fastq,
    Fasta,
    Bed,
    Gff,
    Gtf,
}

#[pyclass(eq, get_all)]
#[derive(Clone, PartialEq, Debug)]
pub struct BioTable {
    pub name: String,
    pub format: InputFormat,
    pub path: String,
}

impl fmt::Display for InputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            InputFormat::Parquet => "Parquet",
            InputFormat::Csv => "CSV",
            InputFormat::Bam => "BAM",
            InputFormat::Vcf => "VCF",
            InputFormat::Fastq => "FASTQ",
            InputFormat::Fasta => "FASTA",
            InputFormat::Bed => "BED",
            InputFormat::Gff => "GFF",
            InputFormat::Gtf => "GTF",
            InputFormat::IndexedBam => "INDEXED_BAM",
            InputFormat::IndexedVcf => "INDEXED_VCF",
            InputFormat::Cram => "CRAM",
        };
        write!(f, "{}", text)
    }
}
#[pyclass(name = "ReadOptions")]
#[derive(Clone, Debug)]
pub struct ReadOptions {
    #[pyo3(get, set)]
    pub vcf_read_options: Option<VcfReadOptions>,
    #[pyo3(get, set)]
    pub gff_read_options: Option<GffReadOptions>,
    #[pyo3(get, set)]
    pub fastq_read_options: Option<FastqReadOptions>,
    #[pyo3(get, set)]
    pub bam_read_options: Option<BamReadOptions>,
    #[pyo3(get, set)]
    pub bed_read_options: Option<BedReadOptions>,
}

#[pymethods]
impl ReadOptions {
    #[new]
    #[pyo3(signature = (vcf_read_options=None, gff_read_options=None, fastq_read_options=None, bam_read_options=None, bed_read_options=None))]
    pub fn new(
        vcf_read_options: Option<VcfReadOptions>,
        gff_read_options: Option<GffReadOptions>,
        fastq_read_options: Option<FastqReadOptions>,
        bam_read_options: Option<BamReadOptions>,
        bed_read_options: Option<BedReadOptions>,
    ) -> Self {
        ReadOptions {
            vcf_read_options,
            gff_read_options,
            fastq_read_options,
            bam_read_options,
            bed_read_options,
        }
    }
}

#[pyclass(name = "PyObjectStorageOptions")]
#[derive(Clone, Debug)]
pub struct PyObjectStorageOptions {
    #[pyo3(get, set)]
    pub chunk_size: Option<usize>,
    #[pyo3(get, set)]
    pub concurrent_fetches: Option<usize>,
    #[pyo3(get, set)]
    pub allow_anonymous: bool,
    #[pyo3(get, set)]
    pub enable_request_payer: bool,
    #[pyo3(get, set)]
    pub max_retries: Option<usize>,
    #[pyo3(get, set)]
    pub timeout: Option<usize>,
    #[pyo3(get, set)]
    pub compression_type: String,
}

#[pymethods]
impl PyObjectStorageOptions {
    #[new]
    #[pyo3(signature = (allow_anonymous, enable_request_payer, compression_type, chunk_size=None, concurrent_fetches=None, max_retries=None, timeout=None, ))]
    pub fn new(
        allow_anonymous: bool,
        enable_request_payer: bool,
        compression_type: String,
        chunk_size: Option<usize>,
        concurrent_fetches: Option<usize>,
        max_retries: Option<usize>,
        timeout: Option<usize>,
    ) -> Self {
        PyObjectStorageOptions {
            allow_anonymous,
            enable_request_payer,
            compression_type,
            chunk_size,
            concurrent_fetches,
            max_retries,
            timeout,
        }
    }
}

pub fn pyobject_storage_options_to_object_storage_options(
    options: Option<PyObjectStorageOptions>,
) -> Option<ObjectStorageOptions> {
    options.map(|opts| ObjectStorageOptions {
        chunk_size: opts.chunk_size,
        concurrent_fetches: opts.concurrent_fetches,
        allow_anonymous: opts.allow_anonymous,
        enable_request_payer: opts.enable_request_payer,
        max_retries: opts.max_retries,
        timeout: opts.timeout,
        compression_type: Some(CompressionType::from_string(opts.compression_type)),
    })
}

#[pyclass(name = "FastqReadOptions")]
#[derive(Clone, Debug)]
pub struct FastqReadOptions {
    #[pyo3(get, set)]
    pub thread_num: Option<usize>,
    pub object_storage_options: Option<ObjectStorageOptions>,
}

#[pymethods]
impl FastqReadOptions {
    #[new]
    #[pyo3(signature = (thread_num=None, object_storage_options=None))]
    pub fn new(
        thread_num: Option<usize>,
        object_storage_options: Option<PyObjectStorageOptions>,
    ) -> Self {
        FastqReadOptions {
            thread_num,
            object_storage_options: pyobject_storage_options_to_object_storage_options(
                object_storage_options,
            ),
        }
    }
    #[staticmethod]
    pub fn default() -> Self {
        FastqReadOptions {
            thread_num: Some(1),
            object_storage_options: Some(ObjectStorageOptions {
                chunk_size: Some(1024 * 1024), // 1MB
                concurrent_fetches: Some(4),
                allow_anonymous: false,
                enable_request_payer: false,
                max_retries: Some(5),
                timeout: Some(300), // 300 seconds
                compression_type: Some(CompressionType::AUTO),
            }),
        }
    }
}

#[pyclass(name = "VcfReadOptions")]
#[derive(Clone, Debug)]
pub struct VcfReadOptions {
    #[pyo3(get, set)]
    pub info_fields: Option<Vec<String>>,
    #[pyo3(get, set)]
    pub format_fields: Option<Vec<String>>,
    #[pyo3(get, set)]
    pub thread_num: Option<usize>,
    pub object_storage_options: Option<ObjectStorageOptions>,
}

#[pymethods]
impl VcfReadOptions {
    #[new]
    #[pyo3(signature = (info_fields=None, format_fields=None, thread_num=None, object_storage_options=None))]
    pub fn new(
        info_fields: Option<Vec<String>>,
        format_fields: Option<Vec<String>>,
        thread_num: Option<usize>,
        object_storage_options: Option<PyObjectStorageOptions>,
    ) -> Self {
        VcfReadOptions {
            info_fields,
            format_fields,
            thread_num,
            object_storage_options: pyobject_storage_options_to_object_storage_options(
                object_storage_options,
            ),
        }
    }
    #[staticmethod]
    pub fn default() -> Self {
        VcfReadOptions {
            info_fields: None,
            format_fields: None,
            thread_num: Some(1),
            object_storage_options: Some(ObjectStorageOptions {
                chunk_size: Some(1024 * 1024), // 1MB
                concurrent_fetches: Some(4),
                allow_anonymous: false,
                enable_request_payer: false,
                max_retries: Some(5),
                timeout: Some(300), // 300 seconds
                compression_type: Some(CompressionType::AUTO),
            }),
        }
    }
}

#[pyclass(name = "GffReadOptions")]
#[derive(Clone, Debug)]
pub struct GffReadOptions {
    #[pyo3(get, set)]
    pub attr_fields: Option<Vec<String>>,
    #[pyo3(get, set)]
    pub thread_num: Option<usize>,
    pub object_storage_options: Option<ObjectStorageOptions>,
}

#[pymethods]
impl GffReadOptions {
    #[new]
    #[pyo3(signature = (attr_fields=None, thread_num=None, object_storage_options=None))]
    pub fn new(
        attr_fields: Option<Vec<String>>,
        thread_num: Option<usize>,
        object_storage_options: Option<PyObjectStorageOptions>,
    ) -> Self {
        GffReadOptions {
            attr_fields,
            thread_num,
            object_storage_options: pyobject_storage_options_to_object_storage_options(
                object_storage_options,
            ),
        }
    }
    #[staticmethod]
    pub fn default() -> Self {
        GffReadOptions {
            attr_fields: None,
            thread_num: Some(1),
            object_storage_options: Some(ObjectStorageOptions {
                chunk_size: Some(1024 * 1024), // 1MB
                concurrent_fetches: Some(4),
                allow_anonymous: false,
                enable_request_payer: false,
                max_retries: Some(5),
                timeout: Some(300), // 300 seconds
                compression_type: Some(CompressionType::AUTO),
            }),
        }
    }
}

#[pyclass(name = "BamReadOptions")]
#[derive(Clone, Debug)]
pub struct BamReadOptions {
    #[pyo3(get, set)]
    pub thread_num: Option<usize>,
    pub object_storage_options: Option<ObjectStorageOptions>,
}

#[pymethods]
impl BamReadOptions {
    #[new]
    #[pyo3(signature = (thread_num=None, object_storage_options=None))]
    pub fn new(
        thread_num: Option<usize>,
        object_storage_options: Option<PyObjectStorageOptions>,
    ) -> Self {
        BamReadOptions {
            thread_num,
            object_storage_options: pyobject_storage_options_to_object_storage_options(
                object_storage_options,
            ),
        }
    }
    #[staticmethod]
    pub fn default() -> Self {
        BamReadOptions {
            thread_num: Some(1),
            object_storage_options: Some(ObjectStorageOptions {
                chunk_size: Some(1024 * 1024), // 1MB
                concurrent_fetches: Some(4),
                allow_anonymous: false,
                enable_request_payer: false,
                max_retries: Some(5),
                timeout: Some(300), // 300 seconds
                compression_type: Some(CompressionType::AUTO),
            }),
        }
    }
}

#[pyclass(name = "BedReadOptions")]
#[derive(Clone, Debug)]
pub struct BedReadOptions {
    #[pyo3(get, set)]
    pub thread_num: Option<usize>,
    pub object_storage_options: Option<ObjectStorageOptions>,
}

#[pymethods]
impl BedReadOptions {
    #[new]
    #[pyo3(signature = (thread_num=None, object_storage_options=None))]
    pub fn new(
        thread_num: Option<usize>,
        object_storage_options: Option<PyObjectStorageOptions>,
    ) -> Self {
        BedReadOptions {
            thread_num,
            object_storage_options: pyobject_storage_options_to_object_storage_options(
                object_storage_options,
            ),
        }
    }
    #[staticmethod]
    pub fn default() -> Self {
        BedReadOptions {
            thread_num: Some(1),
            object_storage_options: Some(ObjectStorageOptions {
                chunk_size: Some(1024 * 1024), // 1MB
                concurrent_fetches: Some(4),
                allow_anonymous: false,
                enable_request_payer: false,
                max_retries: Some(5),
                timeout: Some(300), // 300 seconds
                compression_type: Some(CompressionType::AUTO),
            }),
        }
    }
}
