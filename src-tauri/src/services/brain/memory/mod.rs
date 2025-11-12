pub mod cold;
pub mod hot;
pub mod manager;
pub mod warm;

use cold::ColdMemoryService;
use hot::HotMemoryService;
pub use manager::{MemoryManagerService, MemoryManagerServiceError, MemoryRecord};
use warm::WarmMemoryService;
