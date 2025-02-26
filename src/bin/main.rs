#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::IO;
use esp_hal::ledc::{channel, timer, LSGlobalClkSource, Ledc};
use esp_hal::peripherals::Peripherals;
use esp_hal::prelude::*;
use esp_hal::timer::timg::TimerGroup;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize peripherals
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let timer0 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    // Setup GPIO and PWM for servo control
    let io = IO::new(peripherals.GPIO);
    let mut ledc = Ledc::new(peripherals.LEDC, &mut peripherals.SYSTEM);
    let mut timer = timer::LedcTimer::new(
        &mut ledc,
        timer::Number::Timer0,
        &timer::config::Config {
            duty: timer::config::Duty::Duty10Bit,
            clock_source: LSGlobalClkSource::APBClk,
            frequency: 50_u32.kHz().into(), // 50Hz for servos
        },
    );

    let mut pwm_channel = channel::LedcChannel::new(
        &mut ledc,
        channel::Number::Channel0,
        &timer,
        io.pins.gpio4.into_push_pull_output(), // Change pin as needed
    );

    // Function to set servo position
    async fn set_servo_position(channel: &mut channel::LedcChannel, position: u16) {
        let duty = (position as u32 * 1024 / 180) + 26; // Mapping position to duty cycle
        channel.set_duty(duty.min(1023));
    }

    loop {
        set_servo_position(&mut pwm_channel, 0).await;
        Timer::after(Duration::from_secs(1)).await;
        set_servo_position(&mut pwm_channel, 90).await;
        Timer::after(Duration::from_secs(1)).await;
        set_servo_position(&mut pwm_channel, 180).await;
        Timer::after(Duration::from_secs(1)).await;
    }
}