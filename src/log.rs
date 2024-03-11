pub fn write_debug_log(message: String) {
    log::debug!("DEBUG============== -> {}", message);
}

pub fn write_info_log(message: String) {
    log::info!("INFO============== -> {}", message);
}

pub fn write_error_log(message: String) {
    log::error!("ERROR============== -> {}", message);
}
