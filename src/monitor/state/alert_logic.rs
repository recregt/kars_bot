use std::time::Instant;

use super::AlertState;

impl AlertState {
    pub(crate) fn cpu_should_alert(
        &mut self,
        usage: f32,
        threshold: f32,
        cooldown_secs: u64,
        hysteresis: f32,
        now: Instant,
    ) -> bool {
        should_send_alert(
            usage,
            threshold,
            &mut self.cpu_alerting,
            &mut self.last_cpu_alert,
            cooldown_secs,
            hysteresis,
            now,
        )
    }

    pub(crate) fn ram_should_alert(
        &mut self,
        usage: f32,
        threshold: f32,
        cooldown_secs: u64,
        hysteresis: f32,
        now: Instant,
    ) -> bool {
        should_send_alert(
            usage,
            threshold,
            &mut self.ram_alerting,
            &mut self.last_ram_alert,
            cooldown_secs,
            hysteresis,
            now,
        )
    }

    pub(crate) fn disk_should_alert(
        &mut self,
        usage: f32,
        threshold: f32,
        cooldown_secs: u64,
        hysteresis: f32,
        now: Instant,
    ) -> bool {
        should_send_alert(
            usage,
            threshold,
            &mut self.disk_alerting,
            &mut self.last_disk_alert,
            cooldown_secs,
            hysteresis,
            now,
        )
    }
}

fn should_send_alert(
    usage: f32,
    threshold: f32,
    is_alerting: &mut bool,
    last_sent: &mut Option<Instant>,
    cooldown_secs: u64,
    hysteresis: f32,
    now: Instant,
) -> bool {
    if !*is_alerting && usage > threshold {
        *is_alerting = true;
        *last_sent = Some(now);
        return true;
    }

    let clear_threshold = (threshold - hysteresis).max(0.0);
    if *is_alerting && usage <= clear_threshold {
        *is_alerting = false;
        return false;
    }

    if *is_alerting
        && let Some(last) = *last_sent
        && now.duration_since(last).as_secs() >= cooldown_secs
    {
        *last_sent = Some(now);
        return true;
    }

    false
}
