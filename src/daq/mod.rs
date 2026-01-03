use iced::Subscription;
use iced::futures::Stream;
use iced::futures::channel::mpsc;
use iced::futures::sink::SinkExt;
use iced::stream;

pub enum Event {
    Ready(mpsc::Sender<Input>),
    WorkFinished,
    // ...
}

pub enum Input {
    DoSomeWork,
    // ...
}

pub fn some_worker() -> impl Stream<Item = Event> {
    stream::channel(100, async |mut output| {
        // Create channel
        let (sender, mut receiver) = mpsc::channel(100);

        // Send the sender back to the application
        output.send(Event::Ready(sender)).await;

        loop {
            use iced_futures::futures::StreamExt;

            // Read next input sent from `Application`
            let input = receiver.select_next_some().await;

            match input {
                Input::DoSomeWork => {
                    // Do some async work...

                    // Finally, we can optionally produce a message to tell the
                    // `Application` the work is done
                    output.send(Event::WorkFinished).await;
                }
            }
        }
    })
}

pub fn subscription() -> Subscription<Event> {
    Subscription::run(some_worker)
}
