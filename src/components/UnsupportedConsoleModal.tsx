import Modal from '@/components/elements/Modal'

interface Props {
    open: boolean
}

const UnsupportedConsoleModal = ({ open }: Props) => {
    return (
        <Modal open={open} onClose={() => {}}>
            <Modal.Header>
                <Modal.Title>Unsupported Console</Modal.Title>
            </Modal.Header>

            <Modal.Body>
                <Modal.Description>
                    No token was found in your browser. Please log out and log
                    back in to resolve this issue.
                </Modal.Description>
            </Modal.Body>
            <Modal.Actions>
                <Modal.Action>OK</Modal.Action>
            </Modal.Actions>
        </Modal>
    )
}

export default UnsupportedConsoleModal