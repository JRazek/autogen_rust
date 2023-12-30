/// Very general traits for agents.

/// One may imagine a situation, when we do not have text as our communication format.
/// Both ConsumerAgent and ProducerAgent may be implemented to create an agent that accepts and
/// returns any type of messages.
///
/// Example:
///
/// ```ignore
/// struct AudioTranscription;
///
/// impl ConsumerAgent for AudioTranscription {
///    type Mrx = Vec<u8>;
///    type Error = Error;
///
///    async fn receive_message(&mut self, mrx: Self::Mrx) -> Result<(), Self::Error> {
///      // transcribe audio to text
///      Ok(())
///    }
/// }   
///
/// impl ProducerAgent for AudioTranscription {
///   type Mtx = String;
///   type Error = Error;
///
///   async fn send_message(&mut self) -> Result<Self::Mtx, Self::Error> {
///     // get audio from buffered stream
///     Ok("Hello world".to_string())
///   }
/// }
///
/// ```

pub trait ConsumerAgent {
    type Mrx;
    type Error;

    async fn receive_message(&mut self, mrx: Self::Mrx) -> Result<(), Self::Error>;
}

pub trait ProducerAgent {
    type Mtx;
    type Error;

    /// Based on the current state of Agent, reply with a message.
    async fn send_message(&mut self) -> Result<Self::Mtx, Self::Error>;
}

pub trait NamedAgent {
    fn name(&self) -> &str;
}
