use logical_device::LogicalDevice;

pub trait ComputeConnectorTrait<Instance, Adapter, Device, Queue> {
    fn instance(&self) -> &Instance;
    fn adapter(&self) -> &Adapter;
    fn logical_device(&self) -> &LogicalDevice<Device, Queue>;

    fn device<'a>(&'a self) -> &'a Device
    where
        Queue: 'a,
    {
        self.logical_device().device()
    }

    fn queue<'a>(&'a self) -> &'a Queue
    where
        Device: 'a,
    {
        self.logical_device().queue()
    }
}
pub type ComputeConnectorT<Instance, Adapter, Device, Queue> =
    dyn ComputeConnectorTrait<Instance, Adapter, Device, Queue>;
