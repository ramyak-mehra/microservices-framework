use log::{debug, error, info, log_enabled, Level};

trait LoggerTrait {
    fn init();
    fn stop();
    fn get_log_handler();
}
