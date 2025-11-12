use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{
    ArrayRef, AsArray, FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator, StringArray, UInt32Array,
};
use lancedb::arrow::arrow_schema::{DataType, Field, Schema, SchemaRef};
use lancedb::arrow::IntoArrow;

use crate::services::brain::{MemoryManagerServiceError, MemoryRecord};
use crate::services::EMBEDDING_VECTOR_DIMENSION;

pub fn extract_memory_record_from_batch(
    batch: &RecordBatch,
    row_idx: usize,
) -> Result<MemoryRecord, MemoryManagerServiceError> {
    let id_array = batch
        .column_by_name("id")
        .ok_or_else(|| MemoryManagerServiceError::LanceDbError("Column 'id' not found".to_string()))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| MemoryManagerServiceError::LanceDbError("Failed to downcast 'id' column".to_string()))?;
    let id = id_array.value(row_idx).to_string();

    let vector_array = batch
        .column_by_name("vector")
        .ok_or_else(|| MemoryManagerServiceError::LanceDbError("Column 'vector' not found".to_string()))?;

    let vector = extract_vector_from_array(vector_array, row_idx)?;

    let metadata_array = batch
        .column_by_name("metadata")
        .ok_or_else(|| MemoryManagerServiceError::LanceDbError("Column 'metadata' not found".to_string()))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| MemoryManagerServiceError::LanceDbError("Failed to downcast 'metadata' column".to_string()))?;
    let metadata_str = metadata_array.value(row_idx);
    let metadata: HashMap<String, String> = serde_json::from_str(metadata_str)?;

    let created_at_array = batch
        .column_by_name("created_at")
        .ok_or_else(|| MemoryManagerServiceError::LanceDbError("Column 'created_at' not found".to_string()))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| MemoryManagerServiceError::LanceDbError("Failed to downcast 'created_at' column".to_string()))?;
    let created_at = created_at_array.value(row_idx).to_string();

    let updated_at_array = batch
        .column_by_name("updated_at")
        .ok_or_else(|| MemoryManagerServiceError::LanceDbError("Column 'updated_at' not found".to_string()))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| MemoryManagerServiceError::LanceDbError("Failed to downcast 'updated_at' column".to_string()))?;
    let updated_at = updated_at_array.value(row_idx).to_string();

    let last_accessed_at = batch
        .column_by_name("last_accessed_at")
        .and_then(|col| col.as_any().downcast_ref::<StringArray>())
        .map(|arr| arr.value(row_idx).to_string())
        .unwrap_or_default();

    let access_count = batch
        .column_by_name("access_count")
        .and_then(|col| col.as_any().downcast_ref::<arrow::array::UInt32Array>())
        .map(|arr| arr.value(row_idx))
        .unwrap_or(0);

    Ok(MemoryRecord {
        id,
        vector,
        metadata,
        created_at,
        updated_at,
        last_accessed_at,
        access_count,
    })
}

fn extract_vector_from_array(array: &ArrayRef, row_idx: usize) -> Result<Vec<f32>, MemoryManagerServiceError> {
    if let Some(list_array) = array.as_fixed_size_list_opt() {
        let values = list_array.value(row_idx);
        if let Some(float_array) = values.as_any().downcast_ref::<Float32Array>() {
            let vec: Vec<f32> = (0..float_array.len()).map(|i| float_array.value(i)).collect();
            return Ok(vec);
        }
    }

    if let Some(list_array) = array.as_list_opt::<i32>() {
        let values = list_array.value(row_idx);
        if let Some(float_array) = values.as_any().downcast_ref::<Float32Array>() {
            let vec: Vec<f32> = (0..float_array.len()).map(|i| float_array.value(i)).collect();
            return Ok(vec);
        }
    }

    Err(MemoryManagerServiceError::LanceDbError(
        "Failed to extract vector from array".to_string(),
    ))
}

pub fn batches_to_memory_records(batches: Vec<RecordBatch>) -> Result<Vec<MemoryRecord>, MemoryManagerServiceError> {
    let mut records = Vec::new();

    for batch in batches {
        for row_idx in 0..batch.num_rows() {
            let record = extract_memory_record_from_batch(&batch, row_idx)?;
            records.push(record);
        }
    }

    Ok(records)
}

pub fn memory_record_to_batches(row: MemoryRecord) -> Result<impl IntoArrow, MemoryManagerServiceError> {
    let schema = memory_record_schema();
    let id_array = StringArray::from(vec![row.id]);

    let vector_values = Float32Array::from(row.vector);
    let vector_array = FixedSizeListArray::new(
        Arc::new(Field::new("item", DataType::Float32, true)),
        EMBEDDING_VECTOR_DIMENSION,
        Arc::new(vector_values),
        None,
    );
    let metadata_to_string: String = serde_json::to_string(&row.metadata)?;

    let metadata_array = StringArray::from(vec![metadata_to_string]);
    let created_at_array = StringArray::from(vec![row.created_at]);
    let updated_at_array = StringArray::from(vec![row.updated_at]);
    let last_accessed_at_array = StringArray::from(vec![row.last_accessed_at]);
    let access_count_array = UInt32Array::from(vec![row.access_count]);

    let batches = RecordBatchIterator::new(
        vec![RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(id_array),
                Arc::new(vector_array),
                Arc::new(metadata_array),
                Arc::new(created_at_array),
                Arc::new(updated_at_array),
                Arc::new(last_accessed_at_array),
                Arc::new(access_count_array),
            ],
        )?]
        .into_iter()
        .map(Ok),
        schema.clone(),
    );

    Ok(Box::new(batches))
}

pub fn memory_record_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                EMBEDDING_VECTOR_DIMENSION,
            ),
            true,
        ),
        Field::new("metadata", DataType::Utf8, true),
        Field::new("created_at", DataType::Utf8, true),
        Field::new("updated_at", DataType::Utf8, true),
        Field::new("last_accessed_at", DataType::Utf8, true),
        Field::new("access_count", DataType::UInt32, true),
    ]))
}
